# Personal Naked60BMP configuration

[RMK](https://rmk.rs/) firmware for a Naked60BMP with a nice!nano v2 and an [Adafruit 5199 dongle](https://www.adafruit.com/product/5199), supporting USB, Bluetooth and a wireless dongle connection. The matrix, layers and controls are defined in [src/keyboard.toml](src/keyboard.toml).

## Build

Build both firmware images in the Nix development shell:

```sh
nix develop
moon run build
```

The UF2 images are in `target/firmware/`. To build just one, run `moon run keyboard/build` or `moon run dongle/build`.

Run `moon run check` for Clippy and formatting checks.

## Flash and pair

Double-tap the nice!nano's reset button, then copy the UF2 image to its bootloader drive. Only flash the keyboard; this image is not for the Raytac dongle. The existing bootloader is retained.

For Bluetooth, pair with `Naked60BMP`. Unplug USB to use Bluetooth, or use the output-toggle key while USB is connected. Bluetooth profile 0 is available through the previous and next profile keys and Vial.

The `ClearEeprom` key resets all persistent settings and reboots. Hold a Bluetooth profile selector for five seconds to clear that profile's bond.

Changing RMK's revision, features or storage schema can reset its persistent settings, including Bluetooth bonds and Vial edits. Export any Vial edits before upgrading, and re-pair hosts if settings are reset.

## Dongle

Flash `naked60bmp.uf2` to the keyboard and `naked60bmp-dongle.uf2` to the dongle. Both images are in `target/firmware/`. Update both images together when changing RMK's revision. To open the dongle's bootloader drive (`MDBT50QBOOT`), hold its button while inserting it into USB, then release the button and copy the dongle UF2.

With the dongle plugged in and the keyboard's USB disconnected, hold the management-layer key and tap **1** (`SwitchToDongle`). The keyboard pairs with the dongle automatically. Tap a Bluetooth profile selector to return to a host. If the keyboard is connected by USB, select wireless output with the output-toggle key.

Hold `SwitchToDongle` for five seconds to clear its bond and seek a dongle again. Replug a previously bonded dongle to open its 30-second pairing window. A dongle without a bond keeps seeking until it pairs.

The dongle's blue LED shows its connection state. A steady light means USB is configured and the keyboard's wireless link is connected. Repeating groups of one flash mean USB has not been configured, two mean USB is suspended, and three mean USB is configured but the wireless link is disconnected. This helps distinguish USB enumeration from wireless failures when testing in BIOS.

## Vial

Open [vial.rocks](https://vial.rocks/) in Chrome, Chromium or Edge, and select `Naked60BMP` over direct USB or `Naked60BMP Dongle` while using the dongle. The dongle relays configuration to the keyboard; ordinary host Bluetooth does not expose the WebHID interface. If prompted to unlock, hold **Escape + 1** until the progress bar completes.

The editor displays the full 5×12 matrix and labels the Bluetooth/output controls. Edits are saved on the keyboard; rebuilding after changing `keyboard.toml` does not necessarily replace a saved layout. To restore the source layout, flash once with `clear_layout = true` in `[storage]`, then remove that option before subsequent builds.

On Linux, the browser also needs permission to open the keyboard's `hidraw` device; see [Vial's udev instructions](https://get.vial.today/manual/linux-udev.html). For NixOS, add this to the system configuration, rebuild, then reconnect the keyboard:

```nix
services.udev.extraRules = ''
  KERNEL=="hidraw*", SUBSYSTEM=="hidraw", ATTRS{idVendor}=="4c4b", ATTRS{idProduct}=="6060", MODE="0660", GROUP="users", TAG+="uaccess"
'';
```

The original ZMK firmware is available via `nix build .#firmware`.
