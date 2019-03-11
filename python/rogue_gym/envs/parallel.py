"""Provides ParallelRogueEnv, rogue_gym_core::Runtime wrapper as gym environment"""
from gym import spaces
import json
from typing import Dict, Iterable, List, Tuple, Union
from rogue_gym_python._rogue_gym import ParallelGameState, PlayerState
from .rogue_env import ImageSetting, RogueEnv


class ParallelRogueEnv:
    """Special executor to exec rogue-gym parallely.
    """
    metadata = RogueEnv.metadata
    SYMBOLS = RogueEnv.SYMBOLS
    ACTION_MEANINGS = RogueEnv.ACTION_MEANINGS
    ACTIONS = RogueEnv.ACTIONS
    ACTION_LEN = len(ACTIONS)

    def __init__(
            self,
            config_dicts: Iterable[dict],
            max_steps: int = 1000,
            image_setting: ImageSetting = ImageSetting(),
    ) -> None:
        self.game = ParallelGameState(max_steps, [json.dumps(d) for d in config_dicts])
        self.result = None
        self.max_steps = max_steps
        self.steps = 0
        self.action_space = spaces.discrete.Discrete(self.ACTION_LEN)
        self.observation_space = \
            image_setting.detect_space(*self.game.screen_size(), self.game.symbols())
        self.image_setting = image_setting
        self.states = self.game.states()
        self.num_workers = len(config_dicts)

    def get_key_to_action(self) -> Dict[str, str]:
        return self.ACION_MEANINGS

    def get_configs(self) -> dict:
        config = self.game.dump_config()
        return json.loads(config)

    def step(
            self,
            action: Union[Iterable[int], str]
    ) -> Tuple[List[PlayerState], List[float], List[bool], List[dict]]:
        """
        Do action.
        @param actions(string):
             key board inputs to rogue(e.g. "hjk" or "hh>")
        """
        if isinstance(action, str) and len(action) == self.num_workers:
            action = [ord(c) for c in action]
        else:
            try:
                action = [ord(self.ACTIONS[x]) for x in action]
            except Exception:
                raise ValueError("Invalid action: {}".format(action))
        states = self.game.step(action)
        rewards = [max(0, after.gold - before.gold) for before, after in zip(self.states, states)]
        done = [s.is_terminal for s in states]
        self.states = states
        return self.states, rewards, done, [{}] * self.num_workers

    def reset(self) -> List[PlayerState]:
        """reset game state"""
        self.states = self.game.reset()
        return self.states

    def close(self) -> None:
        self.game.close()

    def seed(self, seeds: List[int]) -> None:
        self.game.seed(seeds)
