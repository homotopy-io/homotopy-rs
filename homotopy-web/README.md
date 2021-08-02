## How to run in debug mode

```sh
# Builds the project and serves it on a local webserver. Auto-reloads when the project changes.
trunk serve
```

## How to build in release mode

```sh
# Builds the project and places it into the `dist` folder.
trunk build --release
```

## What does each file do?

* `Cargo.toml` contains the standard Rust metadata. You put your Rust dependencies in here. You must change this file with your details (name, description, version, authors, categories)

* The `src` folder contains your Rust code.

* The `static` folder contains any files that you want copied as-is into the final build.
