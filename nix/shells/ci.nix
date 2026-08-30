{ pkgs, publishScript, checkScripts, PROMPT ? "" }:
pkgs.mkShell {
  name = "ci";
  shellHook = ''
    export DEVSHELL=ci
    ${PROMPT}
    echo "Scripts:"
    echo "  ${publishScript.name}"
    ${builtins.concatStringsSep "\n" (map (s: "echo \"  ${s.name}\"") checkScripts)}
  '';
  buildInputs = [
    pkgs.rust-bin.stable.latest.default
    pkgs.openssl
    pkgs.pkg-config
    publishScript
  ] ++ checkScripts;
}
