{ pkgs, checkScripts, PROMPT ? "" }:
pkgs.mkShell {
  name = "default";
  shellHook = ''
    export DEVSHELL=default
    ${PROMPT}
    echo "Scripts:"
    ${builtins.concatStringsSep "\n" (map (s: "echo \"  ${s.name}\"") checkScripts)}
  '';
  buildInputs = [
    pkgs.rust-bin.stable.latest.minimal
    pkgs.openssl
    pkgs.pkg-config
  ] ++ checkScripts;
}
