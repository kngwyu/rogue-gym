"""module for wrapper of rogue_gym_core::Runtime as gym environment"""
import gym
import json
import numpy as np
from numpy import ndarray
from typing import Dict, List, Tuple, Union
from rogue_gym_python._rogue_gym import GameState, PlayerState


class RogueEnv(gym.Env):
    metadata = {'render.modes': ['human', 'ascii']}

    # defined in core/src/tile.rs
    SYMBOLS = [
        ' ', '@', '#', '.', '-',
        '%', '+', '^', '!', '?',
        ']', ')', '/', '*', ':',
        '=', ',', 'A', 'B', 'C',
        'D', 'E', 'F', 'G', 'H',
        'I', 'J', 'K', 'L', 'M',
        'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W',
        'X', 'Y', 'Z',
    ]

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
        "s": "SEARCH",
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
        9: "s",
    }

    def __init__(
            self,
            seed: int = None,
            config_path: str = None,
            config_dict: dict = None
    ) -> None:
        """
        @param config_path(string): path to config file
        """
        super().__init__()
        config = None
        if config_dict:
            config = json.dumps(config_dict)
        elif config_path:
            f = open(config_path, 'r')
            config = f.read()
        self.game = GameState(seed, config)
        self.result = None
        self.__cache()

    def __cache(self) -> None:
        self.result = self.game.prev()

    def screen_size(self) -> Tuple[int, int]:
        """
        returns (height, width)
        """
        return self.game.screen_size()

    def channels(self) -> int:
        """
        returns the dimension of feature map
        """
        return self.game.channels()

    def feature_dims(self) -> Tuple[int, int, int]:
        return self.game.feature_dims()

    def get_key_to_action(self) -> Dict[str, str]:
        return self.ACION_MEANINGS

    def get_dungeon(self, is_ascii: bool = True) -> List[str]:
        return self.result.dungeon

    def __step_str(self, actions: str) -> None:
        for act in actions:
            self.game.react(ord(act))

    def state_to_symbol_image(self, state: PlayerState) -> ndarray:
        t = type(state)
        if t is not PlayerState:
            raise ValueError("RogueEnv.state_to_symbol_image is called for " + t)
        return self.game.get_symbol_image(state)

    def step(self, action: Union[int, str]) -> Tuple[PlayerState, float, bool, None]:
        """
        Do action.
        @param actions(string):
             key board inputs to rogue(e.g. "hjk" or "hh>")
        """
        gold_before = self.result.gold
        if type(action) is int:
            s = self.ACTION_MAPPINGS[action]
            self.__step_str(s)
        elif type(action) is str:
            self.__step_str(action)
        else:
            raise ValueError("Invalid action: {}".format(action))
        self.__cache()
        reward = self.result.gold - gold_before
        return self.result, reward, False, None

    def seed(self, seed: int) -> None:
        """
        Set seed.
        This seed is not used till the game is reseted.
        @param seed(int): seed value for RNG
        """
        self.game.set_seed(seed)

    def render(self, mode='human', close: bool = False) -> None:
        """
        STUB
        """
        print(self.result)

    def reset(self) -> None:
        """reset game state"""
        self.game.reset()
        self.__cache()

    def __repr__(self):
        return self.result.__repr__()


