{
  inputs = {
    nixpkgs.url = "github:NickCao/nixpkgs/nixos-unstable-small";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlay ];
          };
        in
        rec {
          devShell = pkgs.mkShell {
            inputsFrom = [ packages.default ];
          };
          packages.default = pkgs.rustPlatform.buildRustPackage {
            name = "carinae";
            src = self;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            nativeBuildInputs = with pkgs;[ rustPlatform.bindgenHook ];
            buildInputs = with pkgs;[ nixVersions.stable boost ];
          };
        }
      );
}
