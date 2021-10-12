# homotopy-web

[Yew](https://yew.rs) web frontend for homotopy.io.

## Requirements

Either use the [`flake.nix`](../flake.nix), or manually install
[`wasm-pack`](https://github.com/rustwasm/wasm-pack) and
[`cargo-make`](https://github.com/sagiegurari/cargo-make).

## How to run in debug mode

```sh
# Builds the project and serves it on localhost:8080.
cargo make serve
```

## How to build in release mode

```sh
# Builds the project and places it into the `dist` folder.
cargo make -p production build
```

## What does each file do?

* `Cargo.toml` contains the standard Rust metadata. You put your Rust dependencies in here. You must change this file with your details (name, description, version, authors, categories)

* `Makefile.toml` contains the
[`cargo-make`](https://github.com/sagiegurari/cargo-make) configuration.

* The `src` folder contains your Rust code.

* The `test` folder contains your Rust tests.

* The `static` folder contains any files that you want copied as-is into the final build.

* The `dist` folder is the complete build artifact.
