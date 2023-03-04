{

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
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
        rust = pkgs.rust-bin.stable."1.64.0".default;
        dr = dependency-refresh.defaultPackage.${system};

        publishScript = pkgs.writeShellScriptBin "crate-publish" ''
          cargo login $1
          cargo publish -p struct-patch-derive
          sleep 5
          cargo publish -p struct-patch
        '';
        updateDependencyScript = pkgs.writeShellScriptBin "update-dependency" ''
          dr ./Cargo.toml
          if [ -f "Cargo.toml.old" ]; then
            rm Cargo.toml.old
            exit 1
          fi
        '';
      in
      with pkgs;
      {
        devShell = mkShell {
          buildInputs = [
            rust
            openssl
            pkg-config

            dr
            publishScript
            updateDependencyScript
          ];
        };
      }
    );
}
