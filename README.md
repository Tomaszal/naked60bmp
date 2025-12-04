# Personal Naked60BMP configuration

- Powered by [ZMK Firmware](https://zmk.dev/)
- Built with [ZMK Nix](https://github.com/lilyinstarlight/zmk-nix)

## Usage

Build and automatically flash firmware:

```
nix run .#flash
```

Build firmware only:

```
nix build
```

Update dependencies:

```
nix flake update
nix run .#update
```
