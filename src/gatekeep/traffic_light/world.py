from typing import Optional, Self
import numpy as np
import gymnasium as gym


class TrafficLightWorld(gym.Env):
    def __init__(self, max_steps: int, video_path: Optional[str] = None):
        super(TrafficLightWorld, self).__init__()
        self.observation_space = gym.spaces.Dict(
            {"traffic_lights": gym.spaces.MultiBinary(4)}
        )
        self.action_space = gym.spaces.MultiBinary(
            4
        )  # action[0] == 1 => North Green, clockwise
        self.state = np.zeros(4)
        self.steps = 0
        self.max_steps = max_steps
        self.video_path = video_path
        self.reset()

    def reset(self):
        self.state = {"traffic_lights": np.random.randint(2, size=4)}
        self.steps = 0
        self.frames = []
        return self.state

    def step(self, action):
        # Update traffic light state based on action
        self.state["traffic_lights"] = action

        self.steps += 1
        done = self.steps >= self.max_steps

        return self.state, 0, done, {}

    def copy(self) -> Self:
        world_copy = TrafficLightWorld(
            max_steps=self.max_steps, video_path=self.video_path
        )
        world_copy.state = self.state.copy()
        world_copy.steps = self.steps
        world_copy.frames = self.frames.copy()
        return world_copy
