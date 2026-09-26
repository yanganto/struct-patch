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
        checkFillerScript = pkgs.writeShellScriptBin "check-filler"
          (builtins.readFile ./nix/scripts/check-filler.sh);
        checkPatchScript = pkgs.writeShellScriptBin "check-patch"
          (builtins.readFile ./nix/scripts/check-patch.sh);
        checkLeakScript = let
          script = pkgs.writeText "check-feature-leak.py" (builtins.readFile ./nix/scripts/check-feature-leak.py);
        in pkgs.writeShellScriptBin "check-feature-leak"
          ''
          if [ $# -eq 0 ]; then
            root="$(git rev-parse --show-toplevel 2>/dev/null)" ||
              { echo "check-feature-leak: not in a git repo; pass a path" >&2; exit 2; }
            set -- "$root/derive/src"
          fi
          exec ${pkgs.python3}/bin/python3 ${script} "$@"
          '';
        testScript = pkgs.writeShellScriptBin "run-test"
          (builtins.readFile ./nix/scripts/run-test.sh);
        PROMPT = ''
          _git_ps1() {
              git rev-parse --is-inside-work-tree &>/dev/null || return
              local branch dirty
              branch=$(git symbolic-ref --short HEAD 2>/dev/null)
              [[ -n $(git status --porcelain) ]] && dirty='*'
              echo "<$branch$dirty>"
          }
          export PS1='\[\e[33m\][$DEVSHELL] \w $(_git_ps1) \$\[\e[0m\] '
          export PS4='\033[31m ⊙ \033[0m'
        '';
      in
      {
        devShells = {
          default = import ./nix/shells/default.nix { inherit pkgs PROMPT; checkScripts = [ checkComplexScript checkFillerScript checkPatchScript testScript checkLeakScript ]; };
          ci = import ./nix/shells/ci.nix { inherit pkgs publishScript PROMPT; checkScripts = [ checkComplexScript checkFillerScript checkPatchScript testScript checkLeakScript ]; };
          no-std = import ./nix/shells/no-std.nix { inherit pkgs PROMPT; checkScripts = [ checkNoStdScript ]; };
        };
          checks = {
            check-feature-leak = pkgs.runCommand "check-feature-leak" { src = self; } ''
              ${checkLeakScript}/bin/check-feature-leak $src/derive/src
              mkdir -p $out
            '';
          };
      }
    );
}
