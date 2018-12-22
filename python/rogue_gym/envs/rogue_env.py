"""Mainly provides RogueEnv, rogue_gym_core::Runtime wrapper as gym environment"""
from enum import Enum, Flag
import gym
from gym import spaces
import json
import numpy as np
from numpy import ndarray
from typing import Dict, List, NamedTuple, Optional, Tuple, Union
from rogue_gym_python._rogue_gym import GameState, PlayerState


class StatusFlag(Flag):
    EMPTY         = 0b000_000_000
    DUNGEON_LEVEL = 0b000_000_001
    HP_CURRENT    = 0b000_000_010
    HP_MAX        = 0b000_000_100
    STR_CURRENT   = 0b000_001_000
    STR_MAX       = 0b000_010_000
    DEFENSE       = 0b000_100_000
    PLAYER_LEVEL  = 0b001_000_000
    EXP           = 0b010_000_000
    HUNGER        = 0b100_000_000
    FULL          = 0b111_111_111

    def count_one(self) -> int:
        s, val = 0, self.value
        for _ in range(9):
            s += val & 1
            val >>= 1
        return s

    def symbol_image(self, state: PlayerState) -> ndarray:
        if not isinstance(state, PlayerState):
            raise TypeError("Needs PlayerState, but {} was given".format(type(state)))
        return state.symbol_image(flag=self.value)

    def symbol_image_with_hist(self, state: PlayerState) -> ndarray:
        if not isinstance(state, PlayerState):
            raise TypeError("Needs PlayerState, but {} was given".format(type(state)))
        return state.symbol_image_with_hist(flag=self.value)

    def gray_image(self, state: PlayerState) -> ndarray:
        if not isinstance(state, PlayerState):
            raise TypeError("Needs PlayerState, but {} was given".format(type(state)))
        return state.gray_image(flag=self.value)

    def gray_image_with_hist(self, state: PlayerState) -> ndarray:
        if not isinstance(state, PlayerState):
            raise TypeError("Needs PlayerState, but {} was given".format(type(state)))
        return state.gray_image_with_hist(flag=self.value)

    def status_vec(self, state: PlayerState) -> List[int]:
        return state.status_vec(flag=self.value)


class DungeonType(Enum):
    GRAY   = 1
    SYMBOL = 2


class ImageSetting(NamedTuple):
    dungeon: DungeonType = DungeonType.SYMBOL
    status: StatusFlag = StatusFlag.FULL
    includes_hist: bool = False

    def dim(self, channels: int) -> int:
        s = channels if self.dungeon == DungeonType.SYMBOL else 1
        s += self.status.count_one()
        s += 1 if self.includes_hist else 0
        return s

    def expand(self, state: PlayerState) -> ndarray:
        if not isinstance(state, PlayerState):
            raise TypeError("Needs PlayerState, but {} was given".format(type(state)))
        if self.dungeon == DungeonType.SYMBOL:
            if self.includes_hist:
                return self.status.symbol_image_with_hist(state)
            else:
                return self.status.symbol_image(state)
        else:
            if self.includes_hist:
                return self.status.gray_image_with_hist(state)
            else:
                return self.status.gray_image(state)


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
            seed: Optional[int] = None,
            config_path: Optional[str] = None,
            config_dict: Optional[dict] = None,
            max_steps: int = 1000,
            image_setting: ImageSetting = ImageSetting(),
    ) -> None:
        super().__init__()
        config = None
        if config_dict:
            config = json.dumps(config_dict)
        elif config_path:
            with open(config_path, 'r') as f:
                config = f.read()
        self.game = GameState(max_steps, seed, config)
        self.result = None
        self.action_space = spaces.discrete.Discrete(self.ACTION_LEN)
        h, w = self.game.screen_size()
        channels = image_setting.dim(self.game.symbols())
        self.observation_space = spaces.box.Box(
            low=0,
            high=1,
            shape=(channels, h, w),
            dtype=np.float32,
        )
        self.image_setting = image_setting
        self.__cache()

    def __cache(self) -> None:
        self.result = self.game.prev()

    def screen_size(self) -> Tuple[int, int]:
        """
        returns (height, width)
        """
        return self.game.screen_size()

    def get_key_to_action(self) -> Dict[str, str]:
        return self.ACION_MEANINGS

    def get_dungeon(self) -> List[str]:
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

    def state_to_image(
            self,
            state: PlayerState,
            setting: Optional[ImageSetting] = None
    ) -> ndarray:
        """Convert PlayerState to 3d array, according to setting or self.expand_setting
        """
        if setting is None:
            setting = self.image_setting
        return setting.expand(state)

    def __step_str(self, actions: str) -> Tuple[int, bool]:
        for i, act in enumerate(actions):
            dead = self.game.react(ord(act))
            if dead:
                return i + 1, True
        return len(actions), False

    def step(self, action: Union[int, str]) -> Tuple[PlayerState, float, bool, dict]:
        """
        Do action.
        @param actions(string):
             key board inputs to rogue(e.g. "hjk" or "hh>")
        """
        gold_before = self.result.gold
        if isinstance(action, int) and action < self.ACTION_LEN:
            s = self.ACTIONS[action]
            step, done = self.__step_str(s)
        elif isinstance(action, str):
            step, done = self.__step_str(action)
        else:
            raise ValueError("Invalid action: {}".format(action))
        self.__cache()
        reward = self.result.gold - gold_before
        return self.result, reward, done, {}

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
        self.__cache()
        return self.result

    def __repr__(self):
        return self.result.__repr__()
