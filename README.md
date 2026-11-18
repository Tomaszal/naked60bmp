# Personal Naked60BMP configuration

[RMK](https://rmk.rs/) firmware for a Naked60BMP with a nice!nano v2, supporting USB and Bluetooth. The matrix, layers and controls are defined in [src/keyboard.toml](src/keyboard.toml).

## Build

Build the keyboard firmware in the Nix development shell:

```sh
nix develop
moon run build
```

The UF2 image is `target/firmware/naked60bmp.uf2`.

Run `moon run check` for Clippy and formatting checks.

## Flash and pair

Double-tap the nice!nano's reset button, then copy the UF2 image to its bootloader drive. The existing bootloader is retained.

For Bluetooth, pair with `Naked60BMP`. Unplug USB to use Bluetooth, or use the output-toggle key while USB is connected. Bluetooth profile 0 is available through the previous and next profile keys and Vial.

The `ClearEeprom` key resets all persistent settings and reboots. Hold a Bluetooth profile selector for five seconds to clear that profile's bond.

Changing RMK's revision, features or storage schema can reset its persistent settings, including Bluetooth bonds and Vial edits. Export any Vial edits before upgrading, and re-pair hosts if settings are reset.

## Vial

Open [vial.rocks](https://vial.rocks/) in Chrome, Chromium or Edge, and select `Naked60BMP` over direct USB. If prompted to unlock, hold **Escape + 1** until the progress bar completes.

The editor displays the full 5×12 matrix and labels the Bluetooth/output controls. Edits are saved on the keyboard; rebuilding after changing `keyboard.toml` does not necessarily replace a saved layout. To restore the source layout, flash once with `clear_layout = true` in `[storage]`, then remove that option before subsequent builds.

On Linux, the browser also needs permission to open the keyboard's `hidraw` device; see [Vial's udev instructions](https://get.vial.today/manual/linux-udev.html). For NixOS, add this to the system configuration, rebuild, then reconnect the keyboard:

```nix
services.udev.extraRules = ''
  KERNEL=="hidraw*", SUBSYSTEM=="hidraw", ATTRS{idVendor}=="4c4b", ATTRS{idProduct}=="6060", MODE="0660", GROUP="users", TAG+="uaccess"
'';
```

The original ZMK firmware is available via `nix build .#firmware`.
