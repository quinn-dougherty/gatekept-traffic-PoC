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
          prism = import ./prism.nix { inherit pkgs; };
          holodeck = import ./holodeck.nix { inherit pkgs; };
          site = import ./site.nix { inherit pkgs lib inps; };
          cli = import ./cli.nix { inherit pkgs; };
        };
      };
    };
}
