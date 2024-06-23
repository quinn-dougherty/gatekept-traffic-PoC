import numpy as np
from tqdm import tqdm
from gatekeep.abcs import SimulationBase
from gatekeep.alpha.types import SimState
from gatekeep.logic.linear_temporal import Proposition
from gatekeep.logic.evaluate import evaluate


def observe(sim: SimulationBase, epsilon: float = 1e-5) -> Proposition:
    if len(sim.vehicles()) == 0:
        return Proposition(SimState.NO_TRAFFIC)

    positions = np.array([vehicle.position for vehicle in sim.vehicles()])
    pairwise_distances = np.linalg.norm(positions[:, None] - positions, axis=-1)
    np.fill_diagonal(
        pairwise_distances, np.inf
    )  # Set diagonal to infinity to avoid self-comparison
    collision = np.any(pairwise_distances < epsilon)

    if collision:
        return Proposition(SimState.CRASH)
    return Proposition(SimState.TRAFFIC_FLOW)


def simulate_trajectories(
    sim, action, num_trajectories: int, len_trajectory: int
) -> list[list[Proposition]]:
    trajectories = []
    for _ in range(num_trajectories):
        trajectory = [observe(sim)]
        for _ in tqdm(range(len_trajectory)):
            _, _, done, _ = sim.step(action)
            trajectory.append(observe(sim))
            if done:
                break
        trajectories.append(trajectory)
    return trajectories


def evaluate_atomic_proposition(prop, observation):
    """
    Implement the logic to evaluate an atomic proposition based on the observation data
    Return a float value between 0 and 1 indicating the degree to which the proposition is satisfied
    """
    match prop:
        case SimState.TRAFFIC_FLOW:
            return 1.0
        case SimState.NO_TRAFFIC:
            return 0.5
        case SimState.CRASH:
            return 0.0
        case _:
            raise ValueError(f"Atomic proposition '{prop}' not found in observation")


def compile_spec(spec, sim):
    def generate_proof_cert(trajectory):
        return {
            "proof_abides": evaluate(
                spec, trajectory, eval_atomic_prop=evaluate_atomic_proposition
            ),
            "action": trajectory,
        }

    return generate_proof_cert
