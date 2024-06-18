{
  outputs = { self, ... } @ inputs:
    inputs.flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ (import inputs.rust-overlay) ];
        };
        lib = pkgs.lib;
        rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rust-toolchain;

        rust-src = craneLib.cleanCargoSource (craneLib.path ./.);
        commonArgs = {
          src = rust-src;
          strictDeps = true;
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          buildInputs = with pkgs; [];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        crate = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });
      in {
        checks = {
          inherit crate;
          crate-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });
          crate-doc = craneLib.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
          });
          crate-fmt = craneLib.cargoFmt {
            src = rust-src;
          };
          crate-audit = craneLib.cargoAudit {
            src = rust-src;
            inherit (inputs) advisory-db;
          };
          crate-nextest = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });
        };
        packages.default = crate;
        apps.default = inputs.flake-utils.lib.mkApp {
          drv = crate;
        };
        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          packages = [
            # Extra dev packages.
          ];
          # RUST_SRC_PATH = "${rust-toolchain}/lib/rustlib/src/rust/library";
          LD_LIBRARY_PATH = lib.makeLibraryPath commonArgs.buildInputs;
        };
      }
    );

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url  = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };
}
