{ inputs, ... }: {
  perSystem = { config, pkgs, ... }: {
    devShells = let
      greeting = "gatekeeper, empirically";
    in {
      py-raw = pkgs.mkShell {
        name = "gatekeep-develop";
        shellHook = "echo ${greeting}";
        buildInputs = (import ./python.nix { inherit pkgs; })
          ++ (with pkgs; [ pkg-config libstdcxx5 zlib ]);
      };
      bootstrap = pkgs.mkShell {
        name = "gatekeep-bootstrap";
        shellHook = ''
          echo "${greeting}"
        '';
        buildInputs = with pkgs; [ rye libclang libgcc ffmpeg-full ];
      };
    };
  };
}
