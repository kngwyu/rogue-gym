"""module for wrapper of rogue_gym_core::Runtime as gym environment"""
import sys
import gym
import numpy as np
from typing import ByteString, Dict, List, Tuple, Union
from rogue_gym_python._rogue_gym import GameState


class RogueResult():
    def update(self, res: Tuple[List[ByteString], Dict, str, np.array]):
        self.dungeon, self.status, self.__status_str, self.feature_map = res

    def gold(self) -> int:
        return self.status['gold']

    def __repr__(self):
        res = ''
        for b in self.dungeon:
            res += b.decode() + '\n'
        res += self.__status_str
        return res


class RogueEnv(gym.Env):
    metadata = {'render.modes': ['human', 'ascii']}
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

    def __init__(self, seed: int = None, config_path: str = None) -> None:
        """
        @param config_path(string): path to config file
        """
        super().__init__()
        config = None
        if config_path:
            f = open(config_path, 'r')
            config = f.read()
        self.game = GameState(config, seed)
        self.result = RogueResult()
        self.__cache()

    def __cache(self) -> None:
        self.result.update(self.game.prev())

    def reset(self):
        """reset game state"""
        self.game.reset()
        self.__cache()

    def __step_str(self, actions: str):
        for act in actions:
            self.game.react(ord(act))

    def step(self, action: Union[int, str]):
        """
        Do action.
        @param actions(string):
             key board inputs to rogue(e.g. "hjk" or "hh>")
        """
        gold_before = self.result.gold()
        if type(action) is int:
            self.game.react(action)
        elif type(action) is str:
            self.__step_str(action)
        else:
            print("Invalid action: ", action)
        self.__cache()
        gold_after = self.result.gold()
        reward = gold_after - gold_before
        return self.result, reward

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
        return self.result

    def show_screen(self, is_ascii=True):
        """
        @param is_ascii(bool): STUB
        """
        print(self.result)

    def render(self, mode='human', close=False):
        if mode == 'ascii':
            return self.get_screen()
        elif mode == 'human':
            print(self.result)

    def get_key_to_action(self):
        return self.ACION_MEANINGS


