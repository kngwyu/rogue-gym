"""module for wrapper of rogue_gym_core::Runtime as gym environment"""
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
        """
        @param config_path(string): path to config file
        """
        super().__init__()
        config = None
        if config_path:
            file = open(config_path, 'r')
            config = file.read()
        self.game = GameState(config_path)
        self.__cache()

    def __cache(self):
        self.cached_dungeon, self.cached_state = self.game.prev()
        
    def reset(self):
        """reset game state"""
        self.game.reset()
        self.__cache()
    
    def step(self, action):
        """
        Do action.
        @param action(one length string): key board input to rogue(e.g. "h" or "j")
        """
        action = ord(action)
        self.cached_dungeon, self.cached_state = self.game.react(action)
        return self.game.react(action)

    def seed(self, seed):
        """
        Set seed.
        This seed is not used till the game is reseted.
        @param seed(int): seed value for RNG
        """
        self.game.set_seed(seed)

    def get_screen(self, is_ascii = True):
        """
        @param is_ascii(bool): STUB
        """
        return self.cached_dungeon

    def show_screen(self, is_ascii = True):
        """
        @param is_ascii(bool): STUB
        """
        for b in self.cached_dungeon:
            print(b)

