# module for wrapper of rogue_gym_core::Runtime as gym environment

import sys
import gym
import numpy as np

try:
    from ._rogue_gym import GameState
except ImportErroor as e:
    raise error.DependencyNotInstalled("{}. (Did you install cargo and rust?)")

class RogueEnv(gym.Env):
    metadata = {'render.modes': ['human', 'ansi']}
    def __init__(self, config_path = None):
        super().__init__()
        config = None
        if config_path:
            file = open(config_path, 'r')
            config = file.read()
        self.game = GameState(config_path)

    def _reset(self):
        self.game.reset()

    def step(self, action):
        map, status = self.game.react(action)
        return map

