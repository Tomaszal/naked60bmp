{
  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    zmk-nix.url = "github:lilyinstarlight/zmk-nix";

    # Follows
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    zmk-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = import inputs.systems;
      perSystem = {
        pkgs,
        lib,
        inputs',
        ...
      }: {
        devShells = {
          default = pkgs.mkShell {
            NIX_PATH = ["nixpkgs=${inputs.nixpkgs}"];

            nativeBuildInputs = with pkgs; [
              alejandra
              nixd
            ];
          };

          zmk = inputs'.zmk-nix.devShells.default;
        };

        packages = let
          firmware = inputs'.zmk-nix.legacyPackages.buildKeyboard {
            name = "zmk-firmware";

            zephyrDepsHash = "sha256-9xdpgu0/A2ZWmJNSlyQ172MGCnkOf/A8WLYp7dvjJ5A=";

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
