{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      config,
      lib,
      ...
    }:
    let
      inps = with pkgs; [
        trunk
        wasm-bindgen-cli
        nodePackages.sass
        binaryen
      ];
    in
    {
      nci = {
        projects.gatekeep = {
          path = "${inputs.self}/src.rs";
          export = true;
          drvConfig.mkDerivation.buildInputs = inps;
        };
        crates = {
          logic = import ./logic.nix { inherit pkgs; };
          abm = import ./abm.nix { inherit pkgs; };
          site = import ./site.nix { inherit pkgs inps; };
        };
      };
    };
}
