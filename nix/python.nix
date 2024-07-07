{ pkgs }:
let
  py-deps =
    ps: with ps; [
      ipython
      jupyter
      pytest
      black
      hypothesis
    ];
  gatekeep-deps =
    ps: with ps; [
      numpy
      scipy
      torch
      matplotlib
      tqdm
      coconut
      gym
      dask
      distributed
      jaxtyping
      einops
      # mesa
    ];
  jax =
    ps: with ps; [
      jax
      jaxlib
      jaxlibWithoutCuda
      ml-dtypes
      jaxtyping
      jaxopt
      einops
    ];
in
[
  (pkgs.python311.withPackages (
    ps:
    builtins.concatLists (
      map (f: f ps) [
        py-deps
        gatekeep-deps
      ]
    )
  ))
  pkgs.sumo
]
