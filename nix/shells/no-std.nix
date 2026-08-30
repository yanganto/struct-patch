{ pkgs, checkNoStdScript, PROMPT ? "" }:
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
    echo "  ${checkNoStdScript.name}"
  '';
  buildInputs = [
    noStdRust
    pkgs.qemu
    checkNoStdScript
  ];
}
