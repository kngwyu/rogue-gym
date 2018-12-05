"""module for wrapper of rogue_gym_core::Runtime as gym environment"""
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


class DungeonType(Enum):
    GRAY   = 1
    SYMBOL = 2


class ExpandSetting(NamedTuple):
    dungeon: DungeonType = DungeonType.SYMBOL
    status: StatusFlag = StatusFlag.FULL
    includes_hist: bool = False

    def _dim(self, channels: int) -> int:
        s = channels if self.dungeon == DungeonType.SYMBOL else 1
        s += self.status.count_one()
        s += 1 if self.includes_hist else 0
        return s


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
            expand_setting: ExpandSetting = ExpandSetting(),
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
        channels = expand_setting._dim(self.game.dungeon_channels())
        self.observation_space = spaces.box.Box(
            low=0,
            high=1,
            shape=(channels, h, w),
            dtype=np.float32,
        )
        self.expand_setting = expand_setting
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

    def expand_state(self, state: PlayerState) -> ndarray:
        if not isinstance(state, PlayerState):
            raise TypeError("Needs PlayerState, but {} was given".format(type(state)))
        exs = self.expand_setting
        if exs.dungeon == DungeonType.SYMBOL:
            if exs.includes_hist:
                return self.game.symbol_image_with_hist(state, flag=exs.status.value)
            else:
                return self.game.symbol_image(state, flag=exs.status.value)
        else:
            if exs.includes_hist:
                return self.game.gray_image_with_hist(state, flag=exs.status.value)
            else:
                return self.game.gray_image(state, flag=exs.status.value)
        
    def symbol_image(self, state: PlayerState, flag: StatusFlag) -> ndarray:
        if not isinstance(state, PlayerState):
            raise TypeError("Needs PlayerState, but {} was given".format(type(state)))
        return self.game.symbol_image(state, flag=flag.value)

    def symbol_image_with_hist(self, state: PlayerState, flag: StatusFlag) -> ndarray:
        if not isinstance(state, PlayerState):
            raise TypeError("Needs PlayerState, but {} was given".format(type(state)))
        return self.game.symbol_image_with_hist(state, flag=flag.value)

    def gray_image(self, state: PlayerState, flag: StatusFlag) -> ndarray:
        if not isinstance(state, PlayerState):
            raise TypeError("Needs PlayerState, but {} was given".format(type(state)))
        return self.game.gray_image(state, flag=flag.value)

    def gray_image_with_hist(self, state: PlayerState, flag: StatusFlag) -> ndarray:
        if not isinstance(state, PlayerState):
            raise TypeError("Needs PlayerState, but {} was given".format(type(state)))
        return self.game.gray_image_with_hist(state, flag=flag.value)

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
