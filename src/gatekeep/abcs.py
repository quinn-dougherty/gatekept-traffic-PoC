from abc import ABC, abstractmethod
import gymnasium as gym


class WorldBase(gym.Env, ABC):
    pass


class SimulationBase(ABC):
    @abstractmethod
    def step(self, action):
        pass

    @abstractmethod
    def reset(self):
        pass

    @abstractmethod
    def render(self):
        pass

    @abstractmethod
    def copy(self):
        pass

    @abstractmethod
    def simulate(self):
        pass


class GatekeeperBase(ABC):
    @abstractmethod
    def run_step(self):
        pass

    def loop(self):
        pass


class ControllerBase(ABC):
    @abstractmethod
    def select_action(self, state):
        pass
