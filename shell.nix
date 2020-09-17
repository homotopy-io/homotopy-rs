{ pkgs ? import <nixpkgs> {} }:
with pkgs;
mkShell {
  buildInputs = [
    nodejs_latest
    gcc wabt wasm-bindgen-cli wasm-pack
    rustc cargo clippy rustfmt rust-analyzer
  ];
}

