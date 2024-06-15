# A toy gatekeeper

proof of concept. 

(super broken as of this commit)

## Usage

Ephemerally,

``` sh
nix develop .#bootstrap
CC=$(which gcc) GCC=$(which g++) rye sync
rye run gatekeep
```

Or install [`rye`](https://rye.astral.sh) yourself

```
rye sync
rye run gatekeep
```

[I don't know](https://github.com/astral-sh/rye/issues/836#issuecomment-2143734800) what the prefix involving `which` is all about, you may or may not need it. 
