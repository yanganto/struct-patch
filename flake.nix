{

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11-small";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, rust-overlay, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        publishScript = pkgs.writeShellScriptBin "crate-publish" ''
          cargo login $1
          cargo publish -p struct-patch-derive || echo "publish struct-patch-derive fail"
          sleep 10
          cargo publish -p struct-patch
        '';
        checkNoStdScript = pkgs.writeShellScriptBin "check-no-std" ''
          set -ex
          cd $(git rev-parse --show-toplevel 2>/dev/null)
          cd examples/no-std-examples
          cargo run --quiet --features=box --bin no-std-box
          cargo run --quiet --features=option --bin no-std-option
        '';
        checkComplexScript = pkgs.writeShellScriptBin "check-complex" ''
          set -ex
          cd $(git rev-parse --show-toplevel 2>/dev/null)
          cd examples/complex-examples
          cargo test --quiet -p substrate
          cargo test --quiet -p catalyst
          cargo test --quiet -p catalyst-src

          echo "Run catatyst test with unsafe features"
          cargo test --quiet -p catalyst --features unsafe
          cargo test --quiet -p catalyst-src --features unsafe
        '';
      in
      with pkgs;
      {
        devShells = let
          noStdRust = rust-bin.stable.latest.default.override {
            targets = [
              "thumbv7m-none-eabi"
            ];
            extensions = [ "rust-src" "llvm-tools-preview" ];
          };
        in
        {
          default = mkShell {
            buildInputs = [
              rust-bin.stable.latest.minimal
              openssl
              pkg-config

              checkComplexScript 
            ];
          };

          ci = mkShell {
            buildInputs = [
              rust-bin.stable.latest.default
              openssl
              pkg-config

              publishScript

              checkComplexScript
            ];
          };

          no-std = mkShell {
            buildInputs = [
              noStdRust
              qemu
              checkNoStdScript
            ];
          };
        };
      }
    );
}
