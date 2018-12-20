"""Provides ParallelRogueEnv, rogue_gym_core::Runtime wrapper as gym environment"""
from gym import spaces
import json
from typing import Dict, Iterable, List, Tuple, Union
from rogue_gym_python._rogue_gym import ParallelGameState, PlayerState
from .rogue_env import RogueEnv


class ParallelRogueEnv:
    """Special executor to exec rogue-gym parallely.
    """
    metadata = RogueEnv.metadata
    SYMBOLS = RogueEnv.SYMBOLS
    ACTION_MEANINGS = RogueEnv.ACTION_MEANINGS
    ACTIONS = RogueEnv.ACTIONS
    ACTION_LEN = len(ACTIONS)

    def __init__(self, config_dicts: Iterable[dict], max_steps: int = 1000) -> None:
        self.game = ParallelGameState(max_steps, list(map(json.dumps, config_dicts)))
        self.result = None
        self.max_steps = max_steps
        self.steps = 0
        self.action_space = spaces.discrete.Discrete(self.ACTION_LEN)
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
        elif isinstance(action, list):
            pass
        else:
            try:
                action = [x for x in action]
            except Exception:
                raise ValueError("Invalid action: {}".format(action))
        states, done = map(list, zip(*self.game.step(action)))
        rewards = [after.gold - before.gold for before, after in zip(self.states, states)]
        self.states = states
        return self.result, rewards, done, [{}] * self.num_workers

    def reset(self) -> List[PlayerState]:
        """reset game state"""
        states = self.game.reset()
        self.states = states
        return self.states
