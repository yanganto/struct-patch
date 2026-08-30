{ pkgs, checkComplexScript, PROMPT ? "" }:
pkgs.mkShell {
  name = "default";
  shellHook = ''
    export DEVSHELL=default
    ${PROMPT}
    echo "Scripts:"
    echo "  ${checkComplexScript.name}"
  '';
  buildInputs = [
    pkgs.rust-bin.stable.latest.minimal
    pkgs.openssl
    pkgs.pkg-config
    checkComplexScript
  ];
}
