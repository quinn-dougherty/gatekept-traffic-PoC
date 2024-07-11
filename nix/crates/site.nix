{
  pkgs,
  lib,
  inps,
}:
{
  targets = {
    "wasm32-unknown-unknown" = {
      default = true;
      # profiles = ["release" "dev"];
      drvConfig.mkDerivation = {
        # add trunk and other dependencies
        nativeBuildInputs = inps;
        # override build phase to build with trunk instead
        buildPhase = lib.mkDefault ''
          export TRUNK_TOOLS_SASS="${pkgs.nodePackages.sass.version}"
          export TRUNK_TOOLS_WASM_BINDGEN="${pkgs.wasm-bindgen-cli.version}"
          echo sass is version $TRUNK_TOOLS_SASS
          echo wasm bindgen is version $TRUNK_TOOLS_WASM_BINDGEN
          export TRUNK_TOOLS_WASM_OPT="version_${pkgs.binaryen.version}"
          export TRUNK_SKIP_VERSION_CHECK="true"
          HOME=$TMPDIR \
            trunk -v build ./site/index.html \
            --dist $out \
            --release \
            ''${cargoBuildFlags:-}
        '';
        # disable install phase because trunk will directly output to $out
        dontInstall = true;
      };
      # profiles.release.runTests = false;
    };
    "x86_64-unknown-linux-gnu".default = false;
  };
}
