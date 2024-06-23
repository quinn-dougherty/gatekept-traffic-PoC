{ inputs, ... }: {
  perSystem = { config, pkgs, ... }: {
    devShells = let greeting = "gatekeeper, empirically";
    in {
      py-raw = pkgs.mkShell {
        name = "gatekeep-develop";
        shellHook = "echo ${greeting}";
        buildInputs = (import ./python.nix { inherit pkgs; })
          ++ (with pkgs; [ pkg-config libstdcxx5 zlib ]);
      };
      default = pkgs.mkShell {
        name = "gatekeep-bootstrap";
        shellHook = ''
          echo "${greeting}"
          export VIRTUAL_ENV=$(pwd)/.venv
          export PATH=$VIRTUAL_ENV/bin:$PATH
          export LIBSUMO_AS_TRACI=1
          export SUMO_HOME=$(which sumo)/share/sumo
        '';
        buildInputs = with pkgs; [ rye libclang libgcc ffmpeg-full sumo ];
      };
    };
  };
}
