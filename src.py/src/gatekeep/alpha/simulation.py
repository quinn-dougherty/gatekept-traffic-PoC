import numpy as np
import matplotlib.pyplot as plt
from gatekeep.abcs import SimulationBase
from gatekeep.alpha.world import TrafficLightWorld


class TrafficLightSim(SimulationBase):
    def __init__(
        self,
        traffic_light_world: TrafficLightWorld,
        road_length: int = 100,
        simulation_time: int = 100,
        dx: float = 1e1,
        dt: float = 1e-1,
    ):
        self.road_length = road_length
        self.simulation_time = simulation_time
        self.dx = dx  # Space step
        self.dt = dt  # Time step

        self.x = np.arange(0, road_length, dx)
        self.t = np.arange(0, simulation_time, dt)

        self.density = np.zeros((len(self.t), len(self.x), 4))  # 4 roads

        # Traffic light positions
        self.light_positions = [int(road_length / 2 / dx)] * 4
        self.traffic_light_world = traffic_light_world

    def fundamental_diagram(self, rho):
        # Greenshield's model
        rho_max = 1.0
        v_free = 1.0
        return v_free * rho * (1 - rho / rho_max)

    def simulate(self):
        """Simplified LWR model with traffic lights."""
        # Initial condition: constant density
        self.density[0, :, :] = 0.3

        for n in range(1, len(self.t)):
            # Get traffic light state
            light_state, _, done, _ = self.traffic_light_world.step(
                np.random.randint(2, size=4)
            )

            # Compute flux
            q = self.fundamental_diagram(self.density[n - 1, :, :])

            # Update density using upwind scheme
            for road in range(4):
                for i in range(1, len(self.x) - 1):
                    if (
                        i == self.light_positions[road]
                        and not light_state["traffic_lights"][road]
                    ):
                        # Red light: no flow
                        self.density[n, i, road] = self.density[n - 1, i, road]
                    else:
                        self.density[n, i, road] = self.density[n - 1, i, road] - (
                            self.dt / self.dx
                        ) * (q[i, road] - q[i - 1, road])

            # Boundary conditions
            self.density[n, 0, :] = self.density[n, 1, :]
            self.density[n, -1, :] = self.density[n, -2, :]

    def plot_results(self):
        fig, axs = plt.subplots(2, 2, figsize=(15, 15))
        roads = ["North", "East", "South", "West"]
        for i, ax in enumerate(axs.flat):
            im = ax.imshow(
                self.density[:, :, i].T,
                aspect="auto",
                origin="lower",
                extent=[0, self.simulation_time, 0, self.road_length],
                cmap="viridis",
            )
            ax.set_xlabel("Time")
            ax.set_ylabel("Position")
            ax.set_title(f"{roads[i]} Road Traffic Density")

        fig.colorbar(im, ax=axs.ravel().tolist(), label="Density")
        plt.tight_layout()
        plt.show()


# Run simulation
max_steps = 1000
traffic_light_world = TrafficLightWorld(max_steps=max_steps)
sim = TrafficSimulation(
    road_length=100,
    simulation_time=100,
    dx=1,
    dt=0.1,
    traffic_light_world=traffic_light_world,
)
sim.simulate()
sim.plot_results()
