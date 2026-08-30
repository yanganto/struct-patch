{ pkgs, checkScripts, PROMPT ? "" }:
let
  noStdRust = pkgs.rust-bin.stable.latest.default.override {
    targets = [ "thumbv7m-none-eabi" ];
    extensions = [ "rust-src" "llvm-tools-preview" ];
  };
in
pkgs.mkShell {
  name = "no-std";
  shellHook = ''
    export DEVSHELL=no-std
    ${PROMPT}
    echo "Scripts:"
    ${builtins.concatStringsSep "\n" (map (s: "echo \"  ${s.name}\"") checkScripts)}
  '';
  buildInputs = [
    noStdRust
    pkgs.qemu
  ] ++ checkScripts;
}
