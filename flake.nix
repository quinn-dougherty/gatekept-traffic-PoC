{
  description = "Gatekeeper, empirically";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    parts.url = "github:hercules-ci/flake-parts";
    fmt = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { self, nixpkgs, parts, fmt }@inputs:
    parts.lib.mkFlake { inherit inputs; } {
      systems =
        [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      imports = [ ./nix/shells.nix fmt.flakeModule ./nix/format.nix ];
      flake.herculesCI.ciSystems = [ "x86_64-linux" ];
    };
}
