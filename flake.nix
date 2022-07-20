{
  description = "homotopy.io rust edition";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    devshell = {
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
      url = "github:numtide/devshell";
    };
    # last version before dream2nix
    nix-cargo-integration = {
      inputs = {
        nixpkgs.follows = "nixpkgs";
        devshell.follows = "devshell";
        rustOverlay.follows = "rust-overlay";
      };
      url = "github:yusdacra/nix-cargo-integration?rev=7fe944f24f1a7014b58ddafbdc8cf1ffae4de1ab";
    };
    naersk = {
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
      # switch to upstream after both of these get merged:
      # https://github.com/nix-community/naersk/pull/167
      # https://github.com/nix-community/naersk/pull/227
      url = "github:NickHu/naersk";
    };
    rust-overlay = {
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
      url = "github:oxalica/rust-overlay";
    };
  };

  outputs = inputs: let
    outputs = inputs.nix-cargo-integration.lib.makeOutputs {
      root = ./.;
      buildPlatform = "crate2nix";
      overrides = {
        build = self: super: {
          # crate2nix insta compatibility fix
          testPreRun = ''
            export INSTA_WORKSPACE_ROOT=$(pwd)
            for file in $INSTA_WORKSPACE_ROOT/tests/snapshots/*; do
              case $(basename $file) in (tests_*) continue;; esac;
              mv $file $INSTA_WORKSPACE_ROOT/tests/snapshots/tests_$(basename $file)
            done
          '';
        };
        shell = self: super: {
          packages =
            super.packages
            ++ (with self.pkgs; [
              (lib.hiPrio rust-bin.nightly.latest.rustfmt)
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              coreutils
            ]);
          commands =
            super.commands
            ++ (with self.pkgs; [
              {package = cargo-make;}
              {package = devserver;}
              {package = gdb;}
              {package = rust-analyzer;}
              {package = wasm-bindgen-cli;}
              {package = nodePackages.firebase-tools;}
            ]);
        };
      };
    };
  in
    outputs
    // inputs.flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import inputs.rust-overlay)];
        pkgs = import inputs.nixpkgs {
          inherit system overlays;
        };
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in rec {
        apps = {
          bench = {
            type = "app";
            program = let
              bench = pkgs.writeShellApplication {
                name = "bench";
                runtimeInputs = [rust];
                text = ''
                  cargo bench | tee output.txt
                '';
              };
            in "${bench}/bin/bench";
          };
          lint = {
            type = "app";
            program = let
              lint = pkgs.writeShellApplication {
                name = "lint";
                runtimeInputs = [pkgs.rust-bin.nightly.latest.rustfmt rust];
                text = ''
                  #shellcheck disable=SC2155
                  export LD_LIBRARY_PATH="${pkgs.zlib}/lib''${LD_LIBRARY_PATH:+LD_LIBRARY_PATH:}"
                  cargo fmt --version
                  cargo fmt --all -- --check
                  cargo clippy -- -D warnings
                '';
              };
            in "${lint}/bin/lint";
          };
          default = {
            type = "app";
            program = toString (pkgs.writeShellScript
            "homotopy-web"
            ''
              ${pkgs.devserver}/bin/devserver --path ${defaultPackage} --header Cross-Origin-Opener-Policy=same-origin --header Cross-Origin-Embedder-Policy=require-corp
            '');
          };
        };
        defaultPackage = let
          rust = pkgs.rust-bin.stable."1.59.0".minimal.override { # TODO: 1.60.0 crashes
            targets = ["wasm32-unknown-unknown"];
          };
          naersk = inputs.naersk.lib."${system}".override {
            cargo = rust;
            rustc = rust;
          };
          meta = builtins.fromTOML (builtins.readFile ./homotopy-web/Cargo.toml);
        in
          naersk.buildPackage {
            inherit (meta.package) name version;
            root = ./.;
            copyLibs = true;
            cargoBuildOptions = opts: opts ++ ["-p" "homotopy-web"];
            CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
            overrideMain = oldAttrs: {
              nativeBuildInputs = oldAttrs.nativeBuildInputs ++ (with pkgs; [wasm-bindgen-cli]);
              postBuild = ''
                ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen --out-dir $out --no-typescript --target web target/wasm32-unknown-unknown/release/homotopy_web.wasm
              '';
              installPhase = ''
                runHook preInstall
                cp -r homotopy-web/static/* $out/
                runHook postInstall
              '';
            };
          };
      }
    );
}
