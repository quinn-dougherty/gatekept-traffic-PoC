import torch
from gatekeep.logic.linear_temporal import (
    Always,
    Eventually,
    And,
    Or,
    Not,
    Tru,
    Proposition,
)
from gatekeep.logic.semantics import always_op, eventually_op, and_op, or_op, not_op


def evaluate(spec, trajectory, eval_atomic_prop):
    def _eval(spec):
        return evaluate(spec, trajectory, eval_atomic_prop)

    if isinstance(spec, Always):
        return always_op(_eval(spec.prop))
    if isinstance(spec, Eventually):
        return eventually_op(_eval(spec.prop))
    if isinstance(spec, And):
        return and_op([_eval(prop) for prop in spec.props])
    if isinstance(spec, Or):
        return or_op([_eval(prop) for prop in spec.props])
    if isinstance(spec, Not):
        return not_op(_eval(spec.prop))
    if isinstance(spec, Tru):
        return torch.tensor(1.0, dtype=torch.float32)
    if isinstance(spec, Proposition):
        return torch.tensor(
            [eval_atomic_prop(spec.name, obs) for obs in trajectory],
            dtype=torch.float32,
        )
    if isinstance(spec, torch.Tensor):
        return spec
    raise ValueError(f"Unsupported specification: {spec}")
