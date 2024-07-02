from pathlib import Path
import gymnasium as gym
import sumo_rl


def create_world(model: str) -> gym.Env:
    """Create a world of traffic lights with sumo."""
    cwd = Path.cwd()
    model_dir = cwd / "src" / "gatekeep" / "traffic_light" / "world" / model
    return gym.make(
        "sumo-rl-v0",
        net_file=str(model_dir / "main.net.xml"),
        route_file=str(model_dir / "main.rou.xml"),
        out_csv_name=str(cwd / "artefacts" / "path_to_output.csv"),
        use_gui=True,
    )
