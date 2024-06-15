from gatekeep.logic.linear_temporal import Proposition, Not, And, Or, Always
from gatekeep.traffic_light.types import WorldState, SimState

# Propositions
n_green = Proposition(WorldState.N)  # North direction is green
e_green = Proposition(WorldState.E)  # East direction is green
s_green = Proposition(WorldState.S)
w_green = Proposition(WorldState.W)

# Safety property
no_perpendicular = Not(
    Or(
        And(n_green, e_green),
        And(n_green, w_green),
        And(s_green, e_green),
        And(s_green, w_green),
    )
)  # no temporal operators.
safety_shield = Always(
    no_perpendicular
)  # North-South and East-West should never be green simultaneously

# spec for gatekeeper
no_traffic = Proposition(SimState.NO_TRAFFIC)
crash = Proposition(SimState.CRASH)
traffic_flow = Proposition(SimState.TRAFFIC_FLOW)

safety = Always(Not(crash))
