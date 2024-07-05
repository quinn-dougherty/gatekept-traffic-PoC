"""Gatekeep, empirically"""

from time import sleep
from gatekeep.alpha import gatekeeper, simulation0 as simulation, world, specification
from gatekeep.controller.random import RandomController
from gatekeep.beta.world.utils import create_world
from gatekeep.gamma import mesa, TrafficModel, agent_portrayal


def single_step_vid(spec):
    """

    Args:
      spec: an LTL `Proposition`

    Returns:
      proof_cert: a certificate
    """
    num_trajectories = 1
    video_path = "gatekept_traffic_light-single.mp4"
    tl_world = world.TrafficLightWorld(max_steps=int(1e1))
    tl_sim = simulation.TrafficLightSim(
        tl_world,
        spawn_rate=1.75e-1,
        max_vehicles=int(2**4),
        vehicle_steps_per_action=int(1e2),
        video_path=video_path,
    )
    controller = RandomController(tl_world)
    tl_gatekeeper = gatekeeper.Gatekeeper(
        tl_sim, spec, controller, num_trajectories=num_trajectories
    )
    next_state, reward, done, proof_cert, trajectories = tl_gatekeeper.run_step()
    tl_gatekeeper.render_and_save_video()
    return reward, proof_cert


def alpha_main() -> int:
    """Main entry point of the alpha program."""
    reward, proof_cert = single_step_vid(specification.safety)
    print(f"reward: {reward} ==== proofcert: {proof_cert}")
    return 0


def beta_main() -> int:
    """Main entry point of the program"""
    world = create_world("single_intersection")
    controller = RandomController(world)
    # sim loop
    next_obs, info = world.reset()
    done = False
    while not done:
        next_obs, reward, terminated, truncated, info = world.step(
            controller.select_action(next_obs)
        )
        done = terminated or truncated
        print(next_obs, reward)
        sleep(1e-3)

    return 0


def main() -> int:
    grid = mesa.visualization.CanvasGrid(agent_portrayal, 11, 11, 500, 500)
    chart = mesa.visualization.ChartModule(
        [{"Label": "Collisions", "Color": "Black"}], data_collector_name="datacollector"
    )

    model_params = {
        "N": mesa.visualization.Slider("Number of cars", 20, 1, 50),
    }

    server = mesa.visualization.ModularServer(
        TrafficModel, [grid, chart], "Traffic Model", model_params
    )

    server.port = 8521
    server.launch()
    return 0
