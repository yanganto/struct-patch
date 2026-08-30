{ pkgs, publishScript, checkScripts, PROMPT ? "" }:
pkgs.mkShell {
  name = "ci";
  shellHook = ''
    export DEVSHELL=ci
    ${PROMPT}
  '';
  buildInputs = [
    pkgs.rust-bin.stable.latest.default
    pkgs.openssl
    pkgs.pkg-config
    publishScript
  ] ++ checkScripts;
}
