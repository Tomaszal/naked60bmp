{
  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    zmk-nix.url = "github:lilyinstarlight/zmk-nix";
    rust-overlay.url = "github:oxalica/rust-overlay";

    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    zmk-nix.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      systems = import inputs.systems;
      perSystem = {
        pkgs,
        lib,
        inputs',
        ...
      }: let
        rustPkgs = import inputs.nixpkgs {
          inherit (pkgs.stdenv.hostPlatform) system;
          overlays = [inputs.rust-overlay.overlays.default];
        };
        rust = rustPkgs.rust-bin.stable.latest.default.override {
          targets = ["thumbv7em-none-eabihf"];
          extensions = ["rust-src"];
        };
        uf2Tools = pkgs.linkFarm "uf2-tools" [
          {
            name = "uf2conv.py";
            path = pkgs.fetchurl {
              url = "https://raw.githubusercontent.com/microsoft/uf2/90e9741f217f5a40c98ba74d663e408041037578/utils/uf2conv.py";
              hash = "sha256-rTa6LWH7LqNxgyJiOSCIKB7uR0xgnfQUKtvuTqPCDyY=";
            };
          }
          {
            name = "uf2families.json";
            path = pkgs.fetchurl {
              url = "https://raw.githubusercontent.com/microsoft/uf2/90e9741f217f5a40c98ba74d663e408041037578/utils/uf2families.json";
              hash = "sha256-nY9WHgUFB8WQkGMAXNb1sVfm/m2x6NJbXZAn1XBkwZQ=";
            };
          }
        ];
      in {
        devShells.zmk = inputs'.zmk-nix.devShells.default;
        devShells.default = pkgs.mkShell {
          CC_thumbv7em_none_eabihf = "${pkgs.gcc-arm-embedded}/bin/arm-none-eabi-gcc";
          AR_thumbv7em_none_eabihf = "${pkgs.gcc-arm-embedded}/bin/arm-none-eabi-ar";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          UF2CONV = "${uf2Tools}/uf2conv.py";

          packages = with pkgs; [
            alejandra
            nixd
            rust
            flip-link
            llvmPackages.llvm
            python3
            gcc-arm-embedded
            moon
          ];
        };

        packages = let
          firmware = inputs'.zmk-nix.legacyPackages.buildKeyboard {
            name = "zmk-firmware";

            zephyrDepsHash = "sha256-SHiCGErcstMH9EbvbQROXIhxFEbMf3AungYu5YvqMEg=";

            src = lib.sourceFilesBySuffices ./. [".yml" ".conf" ".keymap"];

            board = "nice_nano_v2";
            shield = "naked60";
          };
        in {
          inherit firmware;
          default = firmware;
          flash = inputs'.zmk-nix.packages.flash.override {inherit firmware;};
          update = inputs'.zmk-nix.packages.update;
        };
      };
    };
}
