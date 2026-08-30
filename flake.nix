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

        publishScript = pkgs.writeShellScriptBin "crate-publish"
          (builtins.readFile ./nix/scripts/crate-publish.sh);
        checkNoStdScript = pkgs.writeShellScriptBin "check-no-std"
          (builtins.readFile ./nix/scripts/check-no-std.sh);
        checkComplexScript = pkgs.writeShellScriptBin "check-complex"
          (builtins.readFile ./nix/scripts/check-complex.sh);
        PROMPT = ''
          _git_ps1() {
              git rev-parse --is-inside-work-tree &>/dev/null || return
              local branch dirty
              branch=$(git symbolic-ref --short HEAD 2>/dev/null)
              [[ -n $(git status --porcelain) ]] && dirty='*'
              echo "<$branch$dirty>"
          }
          PS1='\[\e[33m\][$DEVSHELL] \w $(_git_ps1) \$\[\e[0m\] '
        '';
      in
      {
        devShells = {
          default = import ./nix/shells/default.nix { inherit pkgs checkComplexScript PROMPT; };
          ci = import ./nix/shells/ci.nix { inherit pkgs publishScript checkComplexScript PROMPT; };
          no-std = import ./nix/shells/no-std.nix { inherit pkgs checkNoStdScript PROMPT; };
        };
      }
    );
}
