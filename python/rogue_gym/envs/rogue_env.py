"""module for wrapper of rogue_gym_core::Runtime as gym environment"""
import sys
import gym
import numpy as np

from rogue_gym_python._rogue_gym import GameState


class RogueEnv(gym.Env):
    metadata = {'render.modes': ['human', 'ascii']}

    def __init__(self, seed=None, config_path=None):
        """
        @param config_path(string): path to config file
        """
        super().__init__()
        config = None
        if config_path:
            f = open(config_path, 'r')
            config = f.read()
        self.game = GameState(config, seed)
        self.__cache()

    def __cache(self):
        self.cached_dungeon, self.cached_state = self.game.prev()

    def __state(self):
        return self.cached_dungeon, self.cached_state

    def reset(self):
        """reset game state"""
        self.game.reset()
        self.__cache()

    def step(self, actions):
        """
        Do action.
        @param actions(string): key board inputs to rogue(e.g. "hjk" or "hh>")
        """
        gold_before = self.cached_state["gold"]
        for act in actions:
            self.game.react(ord(act))
        self.__cache()
        gold_after = self.cached_state["gold"]
        reward = gold_after - gold_before
        return self.cached_dungeon, self.cached_state, reward

    def seed(self, seed):
        """
        Set seed.
        This seed is not used till the game is reseted.
        @param seed(int): seed value for RNG
        """
        self.game.set_seed(seed)

    def get_screen(self, is_ascii=True):
        """
        @param is_ascii(bool): STUB
        """
        return self.cached_dungeon

    def show_screen(self, is_ascii=True):
        """
        @param is_ascii(bool): STUB
        """
        for b in self.cached_dungeon:
            print(b)

    def render(self, mode='human', close=False):
        if mode == 'ascii':
            return self.cached_dungeon
        elif mode == 'human':
            self.game.show_screen()

    # Same as data/keymaps/ai.json
    ACTION_MEANINGS = {
        "h": "MOVE_LEFT",
        "j": "MOVE_UP",
        "k": "MOVE_DOWN",
        "l": "MOVE_RIGHT",
        "n": "MOVE_RIGHTDOWN",
        "b": "MOVE_LEFTDOWN",
        "u": "MOVE_RIGHTUP",
        "y": "MOVE_LEFTDOWN",
        ">": "DOWNSTAIR",
    }

    def get_key_to_action(self):
        return self.ACION_MEANINGS


