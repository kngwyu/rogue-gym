"""module for wrapper of rogue_gym_core::Runtime as gym environment"""
import gym
import numpy as np
from numpy import ndarray
from typing import Any, ByteString, Dict, List, Optional, Tuple, Union
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


class BaseEnv(gym.Env):
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

    ACTION_MAPPINGS = {
        0: "h",
        1: "j",
        2: "k",
        3: "l",
        4: "n",
        5: "b",
        6: "u",
        7: "y",
        8: ">",
    }

    def __init__(
            self,
            seed: int = None,
            config_path: str = None,
            config_str: str = None
    ) -> None:
        """
        @param config_path(string): path to config file
        """
        super().__init__()
        config = None
        if config_str:
            config = config_str
        if not config and config_path:
            f = open(config_path, 'r')
            config = f.read()
        self.game = GameState(seed, config)
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

    def step(self, action: Union[int, str]) -> Tuple[ndarray, float, bool, RogueResult]:
        """
        Do action.
        @param actions(string):
             key board inputs to rogue(e.g. "hjk" or "hh>")
        """
        gold_before = self.result.gold()
        if type(action) is int:
            s = self.ACTION_MAPPINGS[action]
            self.__step_str(s)
        elif type(action) is str:
            self.__step_str(action)
        else:
            print("Invalid action: ", action)
        self.__cache()
        gold_after = self.result.gold()
        reward = gold_after - gold_before
        return self.result.feature_map, reward, False, self.result

    def seed(self, seed: int) -> None:
        """
        Set seed.
        This seed is not used till the game is reseted.
        @param seed(int): seed value for RNG
        """
        self.game.set_seed(seed)

    def get_screen(self, is_ascii: bool = True) -> List[ByteString]:
        """
        @param is_ascii(bool): STUB
        """
        return self.result.dungeon

    def show_screen(self, is_ascii: bool = True) -> None:
        """
        @param is_ascii(bool): STUB
        """
        print(self.result)

    def render(self, mode='human', close: bool = False) -> None:
        print(self.result)

    def get_key_to_action(self) -> Dict[str, str]:
        return self.ACION_MEANINGS


class FirstFloorEnv(BaseEnv):
    def step(self, action: Union[int, str]) -> Tuple[ndarray, float, bool, RogueResult]:
        features, reward, _, res = super().step(action)
        end = False
        if self.result.status["dungeon_level"] == 2:
            end = True
        return self.result.feature_map, reward, end, res


