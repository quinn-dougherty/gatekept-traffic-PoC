from enum import Enum
from typing import Self
import numpy as np


class SimState(Enum):
    TRAFFIC_FLOW = "traffic_flow"
    NO_TRAFFIC = "no_traffic"
    CRASH = "crash"


class WorldState(Enum):
    N = "N"
    E = "E"
    S = "S"
    W = "W"


class Vehicle:
    def __init__(self, position, velocity) -> None:
        self.position = position
        self.velocity = velocity
        pass

    def __repr__(self) -> str:
        return f"Vehicle(position={self.position}, velocity={self.velocity})"

    def __eq__(self, other: Self) -> bool:
        return bool(
            np.all(self.position == other.position)
            and np.all(self.velocity == other.velocity)
        )

    def advance(self) -> None:
        self.position += self.velocity
        pass

    def advance_by(self, velocity) -> None:
        self.position += velocity
        pass

    def advance_by_multiplier(self, multiplier) -> None:
        self.velocity *= multiplier
        self.advance()
        pass

    def stop(self) -> None:
        self.advance_by_multiplier(0)
        pass

    def is_collided(self, other: Self) -> bool:
        return bool(np.linalg.norm(self.position - other.position) > 1 - 2e-2)

    def copy(self) -> "Vehicle":
        return Vehicle(self.position.copy(), self.velocity.copy())

    @classmethod
    def from_direction(cls, direction: int, delta: float) -> "Vehicle":
        match direction:
            case 0:  # S
                position = np.array([0.5 - delta, 0])
                velocity = np.array([0, 0.01])
            case 1:  # E
                position = np.array([1, 0.5 + delta])
                velocity = np.array([-0.01, 0])
            case 2:  # N
                position = np.array([0.5 + delta, 1])
                velocity = np.array([0, -0.01])
            case 3:  # W
                position = np.array([0, 0.5 - delta])
                velocity = np.array([0.01, 0])
        return cls(position, velocity)
