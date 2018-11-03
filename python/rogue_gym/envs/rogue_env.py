"""module for wrapper of rogue_gym_core::Runtime as gym environment"""
import gym
from gym import spaces
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
        ">": "DOWNSTAIR",
        "s": "SEARCH",
    }

    ACTIONS = [
        "h", "j", "k", "l", "n",
        "b", "u", "y", ">", "s",
    ]

    ACTION_LEN = len(ACTIONS)

    def __init__(
            self,
            seed: int = None,
            config_path: str = None,
            config_dict: dict = None,
            max_steps: int = 1000,
            input_channels: int = None,
    ) -> None:
        """
        @param config_path(string): path to config file
        """
        super().__init__()
        config = None
        if config_dict:
            config = json.dumps(config_dict)
        elif config_path:
            with open(config_path, 'r') as f:
                config = f.read()
        self.game = GameState(seed, config)
        self.result = None
        self.max_steps = max_steps
        self.steps = 0
        self.action_space = spaces.discrete.Discrete(self.ACTION_LEN)
        h, w = self.game.screen_size()
        if input_channels:
            self.observation_space = spaces.box.Box(
                low=0,
                high=1,
                shape=(input_channels, h, w),
                dtype=np.float32,
            )
        else:
            # By default, symbol image channels is used
            ipc = self.game.channels()
            self.observation_space = spaces.box.Box(
                low=0,
                high=1,
                shape=(ipc, h, w),
                dtype=np.float32,
            )
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
        return (self.channels(), *self.screen_size)

    def get_key_to_action(self) -> Dict[str, str]:
        return self.ACION_MEANINGS

    def get_dungeon(self, is_ascii: bool = True) -> List[str]:
        return self.result.dungeon

    def get_config(self) -> dict:
        config = self.game.dump_config()
        return json.loads(config)

    def save_config(self, fname: str) -> None:
        with open(fname, 'w') as f:
            f.write(self.game.dump_config())

    def save_actions(self, fname: str) -> None:
        with open(fname, 'w') as f:
            f.write(self.game.dump_history())

    def symbol_image(self, state: PlayerState) -> ndarray:
        if not isinstance(state, PlayerState):
            raise TypeError("Needs PlayerState, but {} was given".format(type(state)))
        return self.game.get_symbol_image(state)

    def symbol_image_with_hist(self, state: PlayerState) -> ndarray:
        if not isinstance(state, PlayerState):
            raise TypeError("Needs PlayerState, but {} was given".format(type(state)))
        return self.game.get_symbol_image_with_hist(state)

    # TODO: remove this function
    def symbol_image_with_hist_and_level(self, state: PlayerState) -> ndarray:
        if not isinstance(state, PlayerState):
            raise TypeError("Needs PlayerState, but {} was given".format(type(state)))
        return self.game.get_symbol_image_with_hist_and_level(state)

    def gray_image(self, state: PlayerState) -> ndarray:
        if not isinstance(state, PlayerState):
            raise TypeError("Needs PlayerState, but {} was given".format(type(state)))
        return self.game.get_gray_image(state)

    def gray_image_with_hist(self, state: PlayerState) -> ndarray:
        if not isinstance(state, PlayerState):
            raise TypeError("Needs PlayerState, but {} was given".format(type(state)))
        return self.game.get_gray_image_with_hist(state)

    def __step_str(self, actions: str) -> int:
        for act in actions:
            self.game.react(ord(act))
        return len(actions)

    def step(self, action: Union[int, str]) -> Tuple[PlayerState, float, bool, None]:
        """
        Do action.
        @param actions(string):
             key board inputs to rogue(e.g. "hjk" or "hh>")
        """
        if self.steps >= self.max_steps:
            return self.result, 0., True, None
        gold_before = self.result.gold
        if isinstance(action, int) and action < self.ACTION_LEN:
            s = self.ACTIONS[action]
            self.steps += self.__step_str(s)
        elif isinstance(action, str):
            self.steps += self.__step_str(action)
        else:
            raise ValueError("Invalid action: {}".format(action))
        self.__cache()
        reward = self.result.gold - gold_before
        done = self.steps >= self.max_steps
        return self.result, reward, done, None

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

    def reset(self) -> PlayerState:
        """reset game state"""
        self.game.reset()
        self.steps = 0
        self.__cache()
        return self.result

    def __repr__(self):
        return self.result.__repr__()
