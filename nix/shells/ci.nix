{ pkgs, publishScript, checkComplexScript, PROMPT ? "" }:
pkgs.mkShell {
  name = "ci";
  shellHook = ''
    export DEVSHELL=ci
    ${PROMPT}
    echo "Scripts:"
    echo "  ${publishScript.name}"
    echo "  ${checkComplexScript.name}"
  '';
  buildInputs = [
    pkgs.rust-bin.stable.latest.default
    pkgs.openssl
    pkgs.pkg-config
    publishScript
    checkComplexScript
  ];
}
