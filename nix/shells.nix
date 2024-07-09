{ inputs, ... }:
{
  perSystem =
    { config, pkgs, ... }:
    {
      devShells =
        let
          greeting = "gatekeeper, empirically";
          js =
            with pkgs;
            mkShell {
              name = "gatekeep-js-devshell";
              shellHook = "echo ${greeting}";
              buildInputs = [
                corepack
                node2nix
              ];
            };
        in
        {
          inherit js;
          rs = config.nci.outputs.gatekeep.devShell;
          rs-js =
            with pkgs;
            mkShell {
              name = "gatekeep.rs-develop";
              shellHook = "echo ${greeting}";
              inputsFrom = [
                js
                config.nci.outputs.gatekeep.devShell
              ];
            };
          py-raw = pkgs.mkShell {
            name = "gatekeep.py-develop";
            shellHook = "echo ${greeting}";
            buildInputs =
              (import ./python.nix { inherit pkgs; })
              ++ (with pkgs; [
                pkg-config
                libstdcxx5
                zlib
              ]);
          };
          py-bootstrap = pkgs.mkShell {
            name = "gatekeep.py-bootstrap";
            shellHook = ''
              echo "${greeting}"
              export VIRTUAL_ENV=$(pwd)/.venv
              export PATH=$VIRTUAL_ENV/bin:$PATH
              export LIBSUMO_AS_TRACI=1
              export SUMO_HOME=$(which sumo)/share/sumo
            '';
            buildInputs = with pkgs; [
              rye
              libclang
              libgcc
              ffmpeg-full
              sumo
            ];
          };
        };
    };
}
