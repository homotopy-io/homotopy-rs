{
  description = "homotopy.io rust edition";

  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          nativeBuildInputs = with pkgs; [
            rust-bin.nightly.latest.rustfmt
            (
              rust-bin.stable.latest.minimal.override {
                targets = [ "wasm32-unknown-unknown" ];
                extensions = [ "rust-docs" "rust-src" "clippy" ];
              }
            )
            wasm-pack
          ];
        in
          {
            devShell = with pkgs; mkShell {
              buildInputs = nativeBuildInputs ++ [
                cargo-make
                devserver
                rust-analyzer
              ];
              RUST_SRC_PATH = "${rust-bin.stable.latest.rust-src}/lib/rustlib/src/rust/library";
            };
          }
    );
}
