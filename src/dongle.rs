#![no_std]
#![no_main]

use defmt::unwrap;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::mode::Async;
use embassy_nrf::peripherals::{RNG, USBD};
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_nrf::usb::{self, Driver};
use embassy_nrf::{bind_interrupts, pac, rng};
use nrf_mpsl::Flash;
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{self as sdc, mpsl};
use panic_probe as _;
use rmk::config::{DeviceConfig, StorageConfig};
use rmk::core_traits::Runnable;
use rmk::dongle::{Dongle, DongleRouter};
use rmk::storage::new_storage_without_keymap;
use rmk::types::connection::UsbState;
use rmk::usb::UsbTransport;
use rmk::{DefaultPacketPool, PacketPool, run_all};
use static_cell::StaticCell;

struct StatusLed<'a> {
    led: Output<'static>,
    router: &'a DongleRouter,
}

impl Runnable for StatusLed<'_> {
    async fn run(&mut self) -> ! {
        loop {
            let flashes = match rmk::state::current_connection_status().usb {
                UsbState::Configured if self.router.is_connected() => 0,
                UsbState::Configured => 3,
                UsbState::Suspended => 2,
                _ => 1,
            };
            if flashes == 0 {
                self.led.set_low();
                embassy_time::Timer::after_millis(100).await;
                continue;
            }
            for _ in 0..flashes {
                self.led.set_low();
                embassy_time::Timer::after_millis(150).await;
                self.led.set_high();
                embassy_time::Timer::after_millis(150).await;
            }
            embassy_time::Timer::after_millis(1200).await;
        }
    }
}

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<USBD>;
    RNG => rng::InterruptHandler<RNG>;
    EGU0_SWI0 => mpsl::LowPrioInterruptHandler;
    CLOCK_POWER => mpsl::ClockInterruptHandler, usb::vbus_detect::InterruptHandler;
    RADIO => mpsl::HighPrioInterruptHandler;
    TIMER0 => mpsl::HighPrioInterruptHandler;
    RTC0 => mpsl::HighPrioInterruptHandler;
});

#[embassy_executor::task]
async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await
}

fn build_sdc<'d, const N: usize>(
    p: sdc::Peripherals<'d>,
    rng: &'d mut rng::Rng<Async>,
    mpsl: &'d MultiprotocolServiceLayer,
    mem: &'d mut sdc::Mem<N>,
) -> Result<sdc::SoftdeviceController<'d>, sdc::Error> {
    sdc::Builder::new()?
        .support_scan()
        .support_central()
        .support_dle_central()
        .support_phy_update_central()
        .support_le_2m_phy()
        .central_count(1)?
        .buffer_cfg(
            DefaultPacketPool::MTU as u16,
            DefaultPacketPool::MTU as u16,
            4,
            4,
        )?
        .build(p, rng, mpsl, mem)
}

fn ble_addr() -> [u8; 6] {
    let ficr = pac::FICR;
    let high = u64::from(ficr.deviceid(1).read());
    let address = (high << 32 | u64::from(ficr.deviceid(0).read())) | 0x0000_c000_0000_0000;
    unwrap!(address.to_le_bytes()[..6].try_into())
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Use the calibrated RC low-frequency clock; no external 32 kHz crystal is required.
    let p = embassy_nrf::init(Default::default());
    // Adafruit 5199: active-low blue LED on P1.13.
    let led = Output::new(p.P1_13, Level::High, OutputDrive::Standard);
    let mpsl_p =
        mpsl::Peripherals::new(p.RTC0, p.TIMER0, p.TEMP, p.PPI_CH19, p.PPI_CH30, p.PPI_CH31);
    let lfclk_cfg = mpsl::raw::mpsl_clock_lfclk_cfg_t {
        source: mpsl::raw::MPSL_CLOCK_LF_SRC_RC as u8,
        rc_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_CTIV as u8,
        rc_temp_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_TEMP_CTIV as u8,
        accuracy_ppm: 500,
        skip_wait_lfclk_started: mpsl::raw::MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED != 0,
    };
    static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
    static SESSION_MEM: StaticCell<mpsl::SessionMem<1>> = StaticCell::new();
    let mpsl = MPSL.init(unwrap!(MultiprotocolServiceLayer::with_timeslots(
        mpsl_p,
        Irqs,
        lfclk_cfg,
        SESSION_MEM.init(mpsl::SessionMem::new()),
    )));
    spawner.spawn(unwrap!(mpsl_task(&*mpsl)));

    let sdc_p = sdc::Peripherals::new(
        p.PPI_CH17, p.PPI_CH18, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23, p.PPI_CH24,
        p.PPI_CH25, p.PPI_CH26, p.PPI_CH27, p.PPI_CH28, p.PPI_CH29,
    );
    let mut rng = rng::Rng::new(p.RNG, Irqs);
    let mut sdc_mem = sdc::Mem::<8192>::new();
    let sdc = unwrap!(build_sdc(sdc_p, &mut rng, mpsl, &mut sdc_mem));
    let driver = Driver::new(p.USBD, Irqs, HardwareVbusDetect::new(Irqs));

    // Match the keyboard's identity so Vial and the same hidraw permission rule work through the relay.
    let device_config = DeviceConfig {
        vid: 0x4c4b,
        pid: 0x6060,
        manufacturer: "Tomaszal",
        product_name: "Naked60BMP Dongle",
        ..DeviceConfig::default()
    };
    // Shared memory.x stops application code at 0xA0000 and preserves the UF2 bootloader.
    let storage_config = StorageConfig {
        start_addr: 0xA0000,
        num_sectors: 32,
        ..Default::default()
    };
    let flash = Flash::take(mpsl, p.NVMC);
    let mut storage = new_storage_without_keymap(flash, storage_config).await;
    let router = DongleRouter::new();
    let mut dongle = Dongle::new(sdc, ble_addr(), &router);
    let mut usb_transport = UsbTransport::new(driver, device_config).with_dongle_router(&router);
    let mut status_led = StatusLed {
        led,
        router: &router,
    };

    run_all!(usb_transport, dongle, storage, status_led).await;
}
