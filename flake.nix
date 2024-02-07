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
        crate2nix.follows = "crate2nix";
      };
      url = "github:NickHu/nix-cargo-integration";
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
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
  };

  outputs = inputs:
    let
      outputs = inputs.nix-cargo-integration.lib.makeOutputs {
        root = ./.;
        buildPlatform = "crate2nix";
        useCrate2NixFromPkgs = true;
        overrides = {
          build = self: super: {
            # crate2nix insta compatibility fix
            testPreRun = ''
              export INSTA_WORKSPACE_ROOT=$(pwd)
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
                { package = cargo-insta; }
                { package = cargo-make; }
                { package = cargo-nextest; }
                { package = cargo-rr; }
                { package = cargo-watch; }
                { package = sfz; }
                { package = gdb; }
                { package = rust-analyzer; }
                { package = wasm-bindgen-cli; }
                { package = nodePackages.firebase-tools; }
              ]);
          };
        };
      };
    in
    outputs
    // inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        # Binaryen's tests are broken
        binaryen-fix = self: super: {
          binaryen = super.binaryen.overrideAttrs (old: { doCheck = false; });
        };
        overlays = [ (import inputs.rust-overlay) binaryen-fix ];
        pkgs = import inputs.nixpkgs {
          inherit system overlays;
        };
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      rec {
        apps = {
          bench = {
            type = "app";
            program =
              let
                bench = pkgs.writeShellApplication {
                  name = "bench";
                  runtimeInputs = [ rust ];
                  text = ''
                    cargo bench | tee output.txt
                  '';
                };
              in
              "${bench}/bin/bench";
          };
          lint = {
            type = "app";
            program =
              let
                lint = pkgs.writeShellApplication {
                  name = "lint";
                  runtimeInputs = [ pkgs.rust-bin.nightly.latest.rustfmt rust ];
                  text = ''
                    #shellcheck disable=SC2155
                    export LD_LIBRARY_PATH="${pkgs.zlib}/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
                    cargo fmt --version
                    cargo fmt --all -- --check
                    cargo clippy -- -D warnings
                  '';
                };
              in
              "${lint}/bin/lint";
          };
          default = {
            type = "app";
            program = toString (pkgs.writeShellScript
              "homotopy-web"
              ''
                ${pkgs.sfz}/bin/sfz --render-index --coi ${defaultPackage}
              '');
          };
        };
        packages = {
          highs_exported_methods = pkgs.writeTextFile {
            name = "exported_functions.json";
            text =
              ''
                [
                  '_malloc',
                  '_free',
                  '_Highs_call',
                  '_Highs_create',
                  '_Highs_run',
                  '_Highs_destroy',
                  '_Highs_getModelStatus',
                  '_Highs_getSolution',
                  '_Highs_getNumCols',
                  '_Highs_getNumRows',
                  '_Highs_changeObjectiveSense',
                  '_Highs_passMip',
                  '_Highs_passLp',
                  '_Highs_setStringOptionValue',
                  '_Highs_setIntOptionValue',
                  '_Highs_setDoubleOptionValue',
                  '_Highs_setBoolOptionValue'
                ]
              '';
          };
          highs_postjs = pkgs.writeTextFile {
            name = "post.js";
            text =
              ''
                window.Highs_call = Module._Highs_call;
                window.Highs_changeObjectiveSense = Module._Highs_changeObjectiveSense;
                window.Highs_create = Module._Highs_create;
                window.Highs_destroy = Module._Highs_destroy;
                window.Highs_getModelStatus = Module._Highs_getModelStatus;
                window.Highs_getNumCols = Module._Highs_getNumCols;
                window.Highs_getNumRows = Module._Highs_getNumRows;
                window.Highs_getSolution = Module._Highs_getSolution;
                window.Highs_run = Module._Highs_run;
                window.Highs_setBoolOptionValue = Module.cwrap("Highs_setBoolOptionValue","number",["number", "string", "number"]);
                window.Highs_setDoubleOptionValue = Module.cwrap("Highs_setDoubleOptionValue","number",["number", "string", "number"]);
                window.Highs_setIntOptionValue = Module.cwrap("Highs_setIntOptionValue","number",["number", "string", "number"]);
                window.Highs_setStringOptionValue = Module.cwrap("Highs_setIntOptionValue","number",["number", "string", "number"]);

                window.Highs_passLp = function (h,nc,nr,nnz,a_fmt,sense,off,ccost,clow,cup,rlow,rup,astart,aidx,aval) {
                  var pccost = Module._malloc(ccost.length*ccost.BYTES_PER_ELEMENT); Module.HEAP8.set(ccost, pccost);
                  var pclow = Module._malloc(clow.length*clow.BYTES_PER_ELEMENT); Module.HEAP8.set(clow, pclow);
                  var pcup = Module._malloc(cup.length*cup.BYTES_PER_ELEMENT); Module.HEAP8.set(cup, pcup);
                  var prlow = Module._malloc(rlow.length*rlow.BYTES_PER_ELEMENT); Module.HEAP8.set(rlow, prlow);
                  var prup = Module._malloc(rup.length*rup.BYTES_PER_ELEMENT); Module.HEAP8.set(rup, prup);
                  var pastart = Module._malloc(astart.length*astart.BYTES_PER_ELEMENT); Module.HEAP8.set(astart, pastart);
                  var paidx = Module._malloc(aidx.length*aidx.BYTES_PER_ELEMENT); Module.HEAP8.set(aidx, paidx);
                  var paval = Module._malloc(aval.length*aval.BYTES_PER_ELEMENT); Module.HEAP8.set(aval, paval);
                  let ret = Module._Highs_passLp(h,nc,nr,nnz,a_fmt,sense,off,pccost,pclow,pcup,prlow,prup,pastart,paidx,paval);
                  Module._free(pccost); Module._free(pclow); Module._free(pcup); Module._free(prlow);
                  Module._free(prup); Module._free(pastart); Module._free(paidx); Module._free(paval);
                  return ret;
                }
                window.Highs_passMip = function (h,nc,nr,nnz,a_fmt,sense,off,ccost,clow,cup,rlow,rup,astart,aidx,aval,integ) {
                  var pccost = Module._malloc(ccost.length*ccost.BYTES_PER_ELEMENT); Module.HEAP8.set(ccost, pccost);
                  var pclow = Module._malloc(clow.length*clow.BYTES_PER_ELEMENT); Module.HEAP8.set(clow, pclow);
                  var pcup = Module._malloc(cup.length*cup.BYTES_PER_ELEMENT); Module.HEAP8.set(cup, pcup);
                  var prlow = Module._malloc(rlow.length*rlow.BYTES_PER_ELEMENT); Module.HEAP8.set(rlow, prlow);
                  var prup = Module._malloc(rup.length*rup.BYTES_PER_ELEMENT); Module.HEAP8.set(rup, prup);
                  var pastart = Module._malloc(astart.length*astart.BYTES_PER_ELEMENT); Module.HEAP8.set(astart, pastart);
                  var paidx = Module._malloc(aidx.length*aidx.BYTES_PER_ELEMENT); Module.HEAP8.set(aidx, paidx);
                  var paval = Module._malloc(aval.length*aval.BYTES_PER_ELEMENT); Module.HEAP8.set(aval, paval);
                  var pinteg = Module._malloc(integ.length*integ.BYTES_PER_ELEMENT); Module.HEAP8.set(integ, pinteg);
                  let ret = Module._Highs_passMip(h,nc,nr,nnz,a_fmt,sense,off,pccost,pclow,pcup,prlow,prup,pastart,paidx,paval,pinteg);
                  Module._free(pccost); Module._free(pclow); Module._free(pcup); Module._free(prlow);
                  Module._free(prup); Module._free(pastart); Module._free(paidx); Module._free(paval); Module._free(pinteg);
                  return ret;
                }

                window.Highs_getSolution = function(h,c,r) {
                  let ptr0=Module._malloc(c+8);let ptr1=Module._malloc(c+8);let ptr2=Module._malloc(r+8);let ptr3=Module._malloc(r+8);
                  let ret=Module._Highs_getSolution(h,ptr0+8,ptr1+8,ptr2+8,ptr3+8);
                  let cv=new Uint8Array(Module.HEAPU8.buffer,ptr0+8,c);
                  let cd=new Uint8Array(Module.HEAPU8.buffer,ptr1+8,c);
                  let rv=new Uint8Array(Module.HEAPU8.buffer,ptr2+8,r);
                  let rd=new Uint8Array(Module.HEAPU8.buffer,ptr3+8,r);
                  Module._free(ptr0);Module._free(ptr1);Module._free(ptr2);Module._free(ptr3);
                  return {"ret": ret, "cv": cv, "cd": cd, "rv": rv, "rd": rd};
                };
              '';
          };
          highs = pkgs.buildEmscriptenPackage rec {
            name = "highs";
            version = "0.7.2";
            src = pkgs.fetchFromGitHub {
              owner = "lovasoa";
              repo = "highs-js";
              rev = "89292f2a0a2e2d5635e0e0f1bb4501ecf63bc709";
              sha256 = "sha256-ksWMY9Bas7RzwX1TOl6z6orJmS+09VJV07oAvB1XhT4=";
              fetchSubmodules = true;
            };
            nativeBuildInputs = with pkgs; [ cmake ];
            configurePhase = ''
              runHook preConfigure

              mkdir -p .emscriptencache
              export EM_CACHE=$(pwd)/.emscriptencache
              mkdir -p build
              cd build
              emcmake cmake ../HiGHS -DOPENMP=OFF -DFAST_BUILD=OFF -DSHARED=OFF

              runHook postConfigure
            '';
            buildPhase = ''
              runHook preBuild

              emmake make -j $NIX_BUILD_CORES libhighs
              emcc -O3 \
                      -s EXPORTED_FUNCTIONS="@${packages.highs_exported_methods}" \
                      -s DISABLE_EXCEPTION_CATCHING=0 \
                      -s EXPORTED_RUNTIME_METHODS="['cwrap','HEAPU8']" \
                      -s EXPORT_NAME="createHighsModule" \
                      -s MODULARIZE=1 \
                      -s ALLOW_MEMORY_GROWTH=1 \
                      -flto \
                      --closure 1 \
                      --post-js="${packages.highs_postjs}" \
                      --closure-args=--externs="${packages.highs_postjs}" \
                      lib/*.a -o highs.mjs

              runHook postBuild
            '';
            installPhase = ''
              runHook preInstall

              mkdir -p $out
              install -Dm644 highs.mjs $out/highs.js
              install -Dm644 highs.wasm $out/

              runHook postInstall
            '';
            checkPhase = ''
            '';
          };
        };
        defaultPackage =
          let
            rust = pkgs.rust-bin.stable.latest.minimal.override {
              targets = [ "wasm32-unknown-unknown" ];
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
            cargoBuildOptions = opts: opts ++ [ "-p" "homotopy-web" ];
            CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
            overrideMain = oldAttrs: {
              nativeBuildInputs = oldAttrs.nativeBuildInputs ++ (with pkgs; [ wasm-bindgen-cli ]);
              buildInputs = oldAttrs.buildInputs ++ ([ packages.highs ]);
              postBuild = ''
                ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen --out-dir $out --no-typescript --target web target/wasm32-unknown-unknown/release/homotopy_web.wasm
              '';
              installPhase = ''
                runHook preInstall
                cp -r homotopy-web/static/* $out/
                cp ${packages.highs}/highs.{js,wasm} $out/
                runHook postInstall
              '';
            };
          };
      }
    );
}
