{

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11-small";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    dependency-refresh.url = "github:yanganto/dependency-refresh";
  };

  outputs = { self, rust-overlay, nixpkgs, flake-utils, dependency-refresh }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        dr = dependency-refresh.defaultPackage.${system};

        publishScript = pkgs.writeShellScriptBin "crate-publish" ''
          cargo login $1
          cargo publish -p struct-patch-derive || echo "publish struct-patch-derive fail"
          sleep 10
          cargo publish -p struct-patch
        '';
        checkCatalystScript = pkgs.writeShellScriptBin "check-catalyst" ''
          cd $(git rev-parse --show-toplevel 2>/dev/null)
          cd complex-example
          cargo test -p substrate
        '';
        updateDependencyScript = pkgs.writeShellScriptBin "update-dependency" ''
          dr ./Cargo.toml

          cd no-std-examples
          dr ./Cargo.toml

          cd ../complex-example
          dr ./Cargo.toml

          if [[ -f "Cargo.toml.old" || -f "no-std-examples/Cargo.toml.old" || -f "complex-example/Cargo.toml.old" ]]; then
            rm -f Cargo.toml.old
            rm -f no-std-examples/Cargo.toml.old
            rm -f complex-example/Cargo.toml.old
            exit 1
          fi
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

              checkCatalystScript 
            ];
          };

          ci = mkShell {
            buildInputs = [
              rust-bin.stable.latest.default
              openssl
              pkg-config

              dr
              publishScript
              updateDependencyScript

              checkCatalystScript
            ];
          };

          no-std = mkShell {
            buildInputs = [
              noStdRust 
              qemu
            ];
          };
        };
      }
    );
}
