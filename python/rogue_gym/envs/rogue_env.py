"""Provides RogueEnv, a gym environment which wraps rogue_gym_core::Runtime"""
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
        self.__check_input(state)
        return state.symbol_image(flag=self.value)

    def symbol_image_with_hist(self, state: PlayerState) -> ndarray:
        self.__check_input(state)
        return state.symbol_image_with_hist(flag=self.value)

    def gray_image(self, state: PlayerState) -> ndarray:
        self.__check_input(state)
        return state.gray_image(flag=self.value)

    def gray_image_with_hist(self, state: PlayerState) -> ndarray:
        self.__check_input(state)
        return state.gray_image_with_hist(flag=self.value)

    def status_vec(self, state: PlayerState) -> List[int]:
        self.__check_input(state)
        return state.status_vec(flag=self.value)

    def __check_input(self, state: PlayerState) -> None:
        if not isinstance(state, PlayerState):
            raise TypeError("Needs PlayerState, but {} was given".format(type(state)))


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

    def detect_space(self, h: int, w: int, symbols: int) -> gym.Space:
        return spaces.box.Box(
            low=0,
            high=1,
            shape=(self.dim(symbols), h, w),
            dtype=np.float32,
        )

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
        '.': 'NO_OPERATION',
        'h': 'MOVE_LEFT',
        'j': 'MOVE_UP',
        'k': 'MOVE_DOWN',
        'l': 'MOVE_RIGHT',
        'n': 'MOVE_RIGHTDOWN',
        'b': 'MOVE_LEFTDOWN',
        'u': 'MOVE_RIGHTUP',
        'y': 'MOVE_LEFTDOWN',
        '>': 'DOWNSTAIR',
        's': 'SEARCH',
    }

    ACTIONS = [
        '.', 'h', 'j', 'k', 'l', 'n',
        'b', 'u', 'y', '>', 's',
    ]

    ACTION_LEN = len(ACTIONS)

    def __init__(
            self,
            config_path: Optional[str] = None,
            config_dict: dict = {},
            max_steps: int = 1000,
            image_setting: ImageSetting = ImageSetting(),
            **kwargs,
    ) -> None:
        super().__init__()
        if config_path:
            with open(config_path, 'r') as f:
                config = f.read()
        else:
            config_dict.update(kwargs)
            config = json.dumps(config_dict)
        self.game = GameState(max_steps, config)
        self.result = None
        self.action_space = spaces.discrete.Discrete(self.ACTION_LEN)
        self.observation_space = \
            image_setting.detect_space(*self.game.screen_size(), self.game.symbols())
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

    def replay(self, interval_ms: int = 100) -> None:
        if not hasattr(self.game, 'replay'):
            raise RuntimeError('Currently replay is only supported on UNIX')
        self.game.replay(interval_ms)

    def play_cli(self) -> None:
        if not hasattr(self.game, 'play_cli'):
            raise RuntimeError('CLI playing is only supported on UNIX')
        self.game.play_cli()

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

    def __step_str(self, actions: str) -> int:
        for act in actions:
            self.game.react(ord(act))
        return len(actions)

    def step(self, action: Union[int, str]) -> Tuple[PlayerState, float, bool, dict]:
        """
        Do action.
        @param actions(string):
             key board inputs to rogue(e.g. "hjk" or "hh>")
        """
        gold_before = self.result.gold
        if isinstance(action, str):
            self.__step_str(action)
        else:
            try:
                s = self.ACTIONS[action]
                self.__step_str(s)
            except Exception as e:
                raise ValueError("Invalid action: {} causes {}".format(action, e))
        self.__cache()
        reward = self.result.gold - gold_before
        return self.result, reward, self.result.is_terminal, {}

    def seed(self, seed: int) -> None:
        """
        Set seed.
        This seed is not used till the game is reseted.
        @param seed(int): seed value for RNG
        """
        self.game.set_seed(seed)

    def render(self, mode: str = 'human', close: bool = False) -> None:
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
