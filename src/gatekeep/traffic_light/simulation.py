from functools import reduce
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.patches import Rectangle, Circle
import matplotlib.animation as animation
from gatekeep.traffic_light.types import Vehicle


class TrafficLightSim:
    CRASH_PENALTY = -1e100
    FLOW_REWARD = 1e0
    DELTA = 7.5e-2
    ACCEL = 1.075e0

    def __init__(
        self,
        world,
        spawn_rate: float,
        max_vehicles: int,
        vehicle_steps_per_action: int,
        video_path: str,
    ):
        self.world = world
        self.spawn_rate = spawn_rate
        self.max_vehicles = max_vehicles
        self.vehicle_steps_per_action = vehicle_steps_per_action
        self.video_path = video_path
        self.current_step = 0
        self.reset()
        self.state = self.world.state.copy()
        self.frames = []

    def step(self, action):
        """
        Args:
          action: bitvector of length 4

        Returns:
          state: dict
          reward: float
          done: bool
          info: dict
        """
        # Update traffic light state based on action
        self.world.step(action)
        self.state["traffic_lights"] = self.world.state["traffic_lights"]

        # Simulate vehicle movement for multiple timesteps per action
        reward = 0
        done = False
        for i in range(self.vehicle_steps_per_action):
            reward_step, done_step = self.simulate_vehicles()
            reward += reward_step
            done = done or done_step
            # if i % 1 == 0:
            self.render()

        return self.state, reward, done, {}

    def reset(self):
        self.current_step = 0
        self.state = self.world.reset()
        self.state["vehicle_positions"] = np.zeros((self.max_vehicles, 2))
        self.state["vehicle_velocities"] = np.zeros((self.max_vehicles, 2))
        self.traffic_light_vehicles = [[]] * 4
        return self.state

    def simulate_vehicles(self):
        self.current_step += 1
        # Spawn new vehicles
        spawn = np.random.rand() < self.spawn_rate
        lt_max_vehicles = len(self.vehicles()) < self.max_vehicles
        if spawn and lt_max_vehicles:
            direction = np.random.choice(range(4))
            new_vehicle = Vehicle.from_direction(direction, self.DELTA)
            if new_vehicle not in self.traffic_light_vehicles[direction]:
                self.traffic_light_vehicles[direction].append(new_vehicle)
        # Advance and process preexisting vehicles
        traffic_light_vehicles = self.traffic_light_vehicles.copy()
        reward = 0
        for direction, vehicles_list in enumerate(traffic_light_vehicles):
            for vehicle in vehicles_list:
                self.traffic_light_vehicles[direction].remove(vehicle)
                if self.state["traffic_lights"][direction] > 0:  # Accelerate if green
                    vehicle.advance_by_multiplier(self.ACCEL)
                else:
                    vehicle.stop()  # Stop if red

                offscreen = np.any(vehicle.position < 0) or np.any(vehicle.position > 1)
                if offscreen:
                    reward += self.FLOW_REWARD
                    if vehicle in self.traffic_light_vehicles[direction]:
                        print(vehicle)
                        self.traffic_light_vehicles[direction].remove(vehicle)
                else:
                    perp_i = (direction - 1) % 4
                    for other_vehicle in traffic_light_vehicles[perp_i]:
                        collision = vehicle.is_collided(other_vehicle)
                        if (
                            vehicle in self.traffic_light_vehicles[direction]
                            and vehicle != other_vehicle
                            and collision
                        ):
                            reward += self.CRASH_PENALTY
                            self.traffic_light_vehicles[direction].remove(vehicle)
                            self.traffic_light_vehicles[perp_i].remove(other_vehicle)
                    perp_j = (direction + 1) % 4
                    for other_vehicle in traffic_light_vehicles[perp_j]:
                        collision = vehicle.is_collided(other_vehicle)
                        if (
                            vehicle in self.traffic_light_vehicles[direction]
                            and vehicle != other_vehicle
                            and collision
                        ):
                            reward += self.CRASH_PENALTY
                            self.traffic_light_vehicles[direction].remove(vehicle)
                            self.traffic_light_vehicles[perp_j].remove(other_vehicle)
                if vehicle not in self.traffic_light_vehicles[direction]:
                    self.traffic_light_vehicles[direction].append(vehicle)
        _ = self.update_vehicle_state()
        done = self.current_step >= self.vehicle_steps_per_action

        return reward, done

    def render(self):
        fig, ax = plt.subplots(figsize=(6, 6))
        ax.set_xlim(0, 1)
        ax.set_ylim(0, 1)
        ax.set_aspect("equal")

        # Draw roads
        ax.plot([0.5 - self.DELTA, 0.5 - self.DELTA], [0, 1], color="gray", linewidth=4)
        ax.plot([0, 1], [0.5 - self.DELTA, 0.5 - self.DELTA], color="gray", linewidth=4)
        ax.plot([0.5 + self.DELTA, 0.5 + self.DELTA], [0, 1], color="gray", linewidth=4)
        ax.plot([0, 1], [0.5 + self.DELTA, 0.5 + self.DELTA], color="gray", linewidth=4)

        # Draw traffic lights
        colors = ["green" if state else "red" for state in self.state["traffic_lights"]]
        light_positions = [
            (0.5 - self.DELTA, 0.25),  # North light
            (0.75, 0.5 + self.DELTA),  # East light
            (0.5 + self.DELTA, 0.75),  # South light
            (0.25, 0.5 - self.DELTA),  # West light
        ]
        for i, (color, position) in enumerate(zip(colors, light_positions)):
            ax.add_patch(Circle(position, 0.03, color=color))

        # Draw vehicles
        for vehicle in self.vehicles():
            x, y = vehicle.position
            ax.add_patch(Rectangle((x - 0.02, y - 0.02), 0.04, 0.04, color="blue"))

        plt.tight_layout()

        fig.canvas.draw()
        frame = np.frombuffer(fig.canvas.tostring_rgb(), dtype=np.uint8)
        frame = frame.reshape(fig.canvas.get_width_height()[::-1] + (3,))
        self.frames.append(frame)

        plt.close(fig)  # Close the figure to prevent rendering to the screen

    def close(self):
        if self.video_path is not None:
            frames = np.stack(self.frames)
            plt.close()
            print(f"Saving video to {self.video_path}")
            self.save_video(frames, self.video_path, fps=2**2)

    @staticmethod
    def save_video(frames, path: str, fps: int = 2**4) -> None:
        fig, ax = plt.subplots()
        ax.axis("off")

        def animate(i: int):
            ax.imshow(frames[i])
            return ax

        anim = animation.FuncAnimation(
            fig, animate, frames=len(frames), interval=1000 / fps
        )
        anim.save(path, writer="ffmpeg", fps=fps)
        plt.close(fig)
        pass

    def copy(self):
        sim_copy = TrafficLightSim(
            world=self.world.copy(),
            spawn_rate=self.spawn_rate,
            max_vehicles=self.max_vehicles,
            vehicle_steps_per_action=self.vehicle_steps_per_action,
            video_path=self.video_path,
        )
        sim_copy.reset()
        sim_copy.state = {
            "traffic_lights": self.state["traffic_lights"].copy(),
            "vehicle_positions": self.state["vehicle_positions"].copy(),
            "vehicle_velocities": self.state["vehicle_velocities"].copy(),
        }
        sim_copy.current_step = self.current_step
        sim_copy.traffic_light_vehicles = [
            [
                Vehicle(vehicle.position.copy(), vehicle.velocity.copy())
                for vehicle in vehicles
            ]
            for vehicles in self.traffic_light_vehicles
        ]
        return sim_copy

    def vehicles(self):
        """
        Concatenates all vehicles from all traffic lights, by flattening the list of lists.

        Assumes: no dups in output (without checking)
        Returns:
            vehicles: list[Vehicle]
        """
        return reduce(lambda x, y: x + y, self.traffic_light_vehicles, list())

    def update_vehicle_state(self):
        vehicles = self.vehicles()
        assert len(vehicles) <= self.max_vehicles, f"Too many vehicles: f{vehicles}"
        for idx, vehicle in enumerate(vehicles):
            self.state["vehicle_positions"][idx] = vehicle.position
            self.state["vehicle_velocities"][idx] = vehicle.velocity
        return self.state
