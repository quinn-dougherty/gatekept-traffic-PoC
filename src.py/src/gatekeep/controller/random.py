from gatekeep.abcs import ControllerBase


class RandomController(ControllerBase):
    def __init__(self, env):
        """
        Random action selector

        Args:
          env: a gymnasium environment
        """
        self.env = env

    def select_action(self, state):
        return self.env.action_space.sample()
