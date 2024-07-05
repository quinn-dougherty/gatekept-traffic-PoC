from functools import reduce
from io import BytesIO
import subprocess
from typing import Optional
from pathlib import Path
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.patches import Rectangle, Circle
import matplotlib.animation as animation
from gatekeep.alpha.types import Vehicle
from gatekeep.abcs import SimulationBase


class TrafficLightSim(SimulationBase):
    """
    A sim has a world, a spawn rate, a max number of vehicles, a number of vehicle steps per action, and a video path. It has a current step, a state, and a list of traffic light vehicles. It has a crash penalty, a flow reward, a delta, and an acceleration.
    It can step, reset, simulate, simulate vehicles, render, close, save video, copy, get vehicles, update vehicle state, and copy.
    """

    CRASH_PENALTY = -1e100
    FLOW_REWARD = 1e0
    DELTA = 7.5e-2
    ACCEL = 1 + 7.5e-2

    def __init__(
        self,
        world,
        spawn_rate: float,
        max_vehicles: int,
        vehicle_steps_per_action: int,
        video_path: Optional[Path | str],
    ):
        self.world = world
        self.spawn_rate = spawn_rate
        self.max_vehicles = max_vehicles
        self.vehicle_steps_per_action = vehicle_steps_per_action
        if isinstance(video_path, str):
            assert "/" not in video_path, "video_path should be a filename, not a path"
            assert ".mp4" == video_path[-4:], "video_path should end with .mp4"
        elif isinstance(video_path, Path):
            assert (
                not video_path.is_dir()
            ), "video_path should be a filename, not a directory"
            assert ".mp4" == video_path.suffix, "video_path should end with .mp4"
        self.video_path = (
            Path("videos") / video_path if video_path is not None else None
        )
        self.current_step = 0
        self.reset()
        self.state = self.world.state.copy()
        self.crash_events = []
        self.frames = []

        self.light_positions = [
            (0.5 + self.DELTA, 0.75),  # North light
            (0.75, 0.5 + self.DELTA),  # East light
            (0.5 - self.DELTA, 0.25),  # South light
            (0.25, 0.5 - self.DELTA),  # West light
        ]

    def step(self, action: np.ndarray):
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
        state, reward, done, _ = self.world.step(action)
        self.state["traffic_lights"] = state["traffic_lights"]

        # Simulate vehicle movement for multiple timesteps per action
        for i in range(self.vehicle_steps_per_action):
            reward_step, done_step = self.simulate_vehicles()
            reward += reward_step
            done = done or done_step
            # if i % 10 == 0:
            self.render()

        return self.state, reward, done, {}

    def reset(self):
        self.current_step = 0
        self.state = self.world.reset()
        self.state["vehicle_positions"] = np.zeros((self.max_vehicles, 2))
        self.state["vehicle_velocities"] = np.zeros((self.max_vehicles, 2))
        self.trafficlight_vehicles_map = [[]] * 4
        self.crash_events = []
        return self.state

    def simulate(self) -> tuple[float, bool]:
        return self.simulate_vehicles()

    def simulate_vehicles(self) -> tuple[float, bool]:
        self.current_step += 1
        # Spawn new vehicles
        spawn = np.random.rand() < self.spawn_rate
        lt_max_vehicles = len(self.vehicles()) < self.max_vehicles
        if spawn and lt_max_vehicles:
            direction = np.random.choice(range(4))
            # print(f"CURRENT_STEP: {self.current_step} &&& DIRECTION: {direction}")
            new_vehicle = Vehicle.from_direction(direction, self.DELTA)
            if new_vehicle not in self.trafficlight_vehicles_map[direction]:
                self.trafficlight_vehicles_map[direction].append(new_vehicle)
        reward = 0

        def is_vehicle_before_light(vehicle: Vehicle) -> list[bool]:
            """
            Args:
              vehicle: a Vehicle
            Returns:
                list[bool]: a list representing if the vehicle is before the light, for each direction
            """
            return [
                vehicle.position[1] > self.light_positions[0][1],  # North
                vehicle.position[0] > self.light_positions[1][0],  # East
                vehicle.position[1] < self.light_positions[2][1],  # South
                vehicle.position[0] < self.light_positions[3][0],  # West
            ]

        # Advance and process preexisting vehicles
        for direction in range(4):
            light = self.state["traffic_lights"][direction]
            vehicles_list = self.trafficlight_vehicles_map[direction]
            for vehicle in vehicles_list:
                self.trafficlight_vehicles_map[direction].remove(vehicle)
                if light == 0 and not is_vehicle_before_light(vehicle)[direction]:
                    # Continue moving if red light and vehicle has passed the light
                    vehicle.advance()
                elif light == 1:
                    # Accelerate if green
                    vehicle.advance_by_multiplier(self.ACCEL)
                else:
                    # Stop if red and vehicle before the light
                    vehicle.stop()
                # self.trafficlight_vehicles_map[direction].append(vehicle)
                offscreen = np.any(vehicle.position < 0) or np.any(vehicle.position > 1)
                # print(f"OFFSCREEN: {offscreen} &&& Vehicle: {vehicle}")
                if offscreen:
                    reward += self.FLOW_REWARD
                    # self.trafficlight_vehicles_map[direction].remove(vehicle)
                else:
                    self.trafficlight_vehicles_map[direction].append(vehicle)
                perp_i = (direction - 1) % 4
                for other_vehicle in self.trafficlight_vehicles_map[perp_i]:
                    if vehicle.is_collided(other_vehicle):
                        reward += self.CRASH_PENALTY
                        self.crash_events.append(vehicle.position)
                        if vehicle in self.trafficlight_vehicles_map[direction]:
                            self.trafficlight_vehicles_map[direction].remove(vehicle)
                        if other_vehicle in self.trafficlight_vehicles_map[perp_i]:
                            self.trafficlight_vehicles_map[perp_i].remove(other_vehicle)
                perp_j = (direction + 1) % 4
                for other_vehicle in self.trafficlight_vehicles_map[perp_j]:
                    if vehicle.is_collided(other_vehicle):
                        reward += self.CRASH_PENALTY
                        self.crash_events.append(vehicle.position)
                        self.trafficlight_vehicles_map[direction].remove(vehicle)
                        self.trafficlight_vehicles_map[perp_j].remove(other_vehicle)
        _ = self.update_vehicle_state()
        done = self.current_step >= self.vehicle_steps_per_action
        return reward, done

    def render(self) -> None:
        dpi = 24
        fig, ax = plt.subplots(figsize=(6, 6), dpi=dpi)
        ax.set_xlim(0, 1)
        ax.set_ylim(0, 1)
        ax.set_aspect("equal")

        # Draw roads
        ax.plot([0.5 - self.DELTA, 0.5 - self.DELTA], [0, 1], color="gray", linewidth=4)
        ax.plot([0, 1], [0.5 - self.DELTA, 0.5 - self.DELTA], color="gray", linewidth=4)
        ax.plot([0.5 + self.DELTA, 0.5 + self.DELTA], [0, 1], color="gray", linewidth=4)
        ax.plot([0, 1], [0.5 + self.DELTA, 0.5 + self.DELTA], color="gray", linewidth=4)

        # Draw traffic lights
        colors = [
            "green" if light > 0 else "red" for light in self.state["traffic_lights"]
        ]

        for color, position in zip(colors, self.light_positions):
            ax.add_patch(Circle(position, 0.03, color=color))

        # Draw vehicles
        for vehicle in self.vehicles():
            x, y = vehicle.position
            ax.add_patch(Rectangle((x - 0.02, y - 0.02), 0.04, 0.04, color="blue"))

        # Draw explosions for crashes
        for crash_position in self.crash_events:
            ax.add_patch(Circle(crash_position, 0.05, color="orange"))

        # Clear crash events after rendering
        self.crash_events = []

        plt.tight_layout()

        fig.canvas.draw()
        ### With buffer is possibly faster.
        # buff = BytesIO()
        # fig.savefig(buff, format="png", dpi=dpi)
        # buff.seek(0)
        # frame = plt.imread(buff)
        # self.frames.append(frame)
        frame = np.frombuffer(fig.canvas.tostring_rgb(), dtype=np.uint8)
        frame = frame.reshape(fig.canvas.get_width_height()[::-1] + (3,))
        self.frames.append(frame)

        plt.close(fig)  # Close the figure to prevent rendering to the screen
        pass

    def close(self):
        if self.video_path is not None:
            frames = np.stack(self.frames)

            plt.close()
            print(f"Saving video to {self.video_path}")
            self.save_video(frames, str(self.video_path), fps=2**2)

    @staticmethod
    def save_video(frames, path: str, fps: int = 2**4) -> None:
        def get_ffmpeg_path():
            try:
                # Run the 'which ffmpeg' command and capture the output
                return (
                    subprocess.check_output("which ffmpeg", shell=True).decode().strip()
                )
            except subprocess.CalledProcessError:
                # 'which' command returned a non-zero exit status (ffmpeg not found)
                return None

        # Get the path to the ffmpeg executable
        ffmpeg_path = get_ffmpeg_path()

        if ffmpeg_path:
            # Set the ffmpeg path in Matplotlib
            plt.rcParams["animation.ffmpeg_path"] = ffmpeg_path
            print(f"FFmpeg path set to: {ffmpeg_path}")
        else:
            print("FFmpeg not found. Please make sure it is installed and accessible.")

        fig, ax = plt.subplots()
        ax.axis("off")

        def animate(i: int):
            ax.imshow(frames[i])
            return ax

        def callback(i: int, n: int) -> None:
            if i % 2**5 == 0:
                print(f"Saving frame {i}/{n}", end="\t")
            pass

        anim = animation.FuncAnimation(
            fig, animate, frames=len(frames), interval=1e3 / fps
        )

        anim.save(path, writer="ffmpeg", fps=fps, codec="h264")
        plt.close(fig)
        pass

    def copy(self) -> "TrafficLightSim":
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
        sim_copy.trafficlight_vehicles_map = [
            [
                Vehicle(vehicle.position.copy(), vehicle.velocity.copy())
                for vehicle in vehicles
            ]
            for vehicles in self.trafficlight_vehicles_map
        ]
        return sim_copy

    def vehicles(self):
        """
        Concatenates all vehicles from all traffic lights, by flattening the list of lists.

        Assumes: no dups in input intersections (without checking)
        Returns:
            vehicles: list[Vehicle]
        """
        return reduce(lambda x, y: x + y, self.trafficlight_vehicles_map, list())

    def update_vehicle_state(self):
        vehicles = self.vehicles()
        assert len(vehicles) <= self.max_vehicles, f"Too many vehicles: {len(vehicles)}"
        for idx, vehicle in enumerate(vehicles):
            self.state["vehicle_positions"][idx] = vehicle.position
            self.state["vehicle_velocities"][idx] = vehicle.velocity
        return self.state
