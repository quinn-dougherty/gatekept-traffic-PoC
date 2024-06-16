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

## Goals and directions

One way we might make AI go well is to lock down its outputs behind some buffer that can accept or reject actions, only passing safe actions along to the world. This is one part of the “gatekeeper” vision. 

To my knowledge, there’s no empirical codebase people can disagree about being or not being a gatekeeper. I choose a minimal toy problem (traffic lights and traffic) and a basic specification language (LTL) to build toward a notion of proof certificate. The next directions are 
- Interpretable data structures for proof certs. Right now I basically only have the float that the LTL evaluator returns, which is deeply unsatisfying. 
    - If we interpret trajectories as state machine trajectories, we could adopt a traditional model checking flavor and try off the shelf solutions. 
- Potentially some tricks like monte carlo tree search to make it not take forever to run. 
- Smoothing the gatekeeper API surface, make a library only slightly harder to use than gymnasium with its step/reset obligations but it ships with a temporal logic for specifying properties.  
- The left half of the above gatekeeper diagram, instead of a spec that falls out of the sky. 
- Train models (rllib), instead of random controller
- Find LLM analogues
- Write up and post to spark debates
- Scale to more and bigger problems
- Laboratory in minetest or azerothcore. We should be able to simulate most of the theory of change in an open world video game.  
- Compare/contrast reward function design with spec design, to see if assurances are more interpretable in this regime. 

Of these, I’m most excited about data structures for proof certs and the potential to plug in an off the shelf model checker, and secondly shipping gatekeeping as a library. Unfortunately, I’m skeptical that the laboratory in minecraft is achievable by myself working alone, I may not be able to do it without another dev. 
