## Usage

Ephemerally,

```sh
nix develop
CC=$(which gcc) GCC=$(which g++) rye sync
rye run gatekeep
```

Or install [`rye`](https://rye.astral.sh) and [`sumo`](https://sumo.dlr.de/docs/Installing/index.html) yourself

```
sudo add-apt-repository ppa:sumo/stable
sudo apt-get update
sudo apt-get install sumo sumo-tools sumo-doc

rye sync
rye run gatekeep
```

[I don't know](https://github.com/astral-sh/rye/issues/836#issuecomment-2143734800) what the prefix involving `which` is all about, you may or may not need it.
