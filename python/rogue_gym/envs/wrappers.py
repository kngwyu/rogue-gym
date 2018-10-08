from .rogue_env import PlayerState, RogueEnv
from typing import Tuple, Union


class FirstFloorEnv(RogueEnv):
    def __init__(self, *args, **kwargs) -> None:
        self.stair_reward = 100.0  # default reward
        if 'stair_reward' in kwargs:
            self.stair_reward = kwargs['stair_reward']
            del kwargs['stair_reward']
        super().__init__(*args, **kwargs)

    def step(self, action: Union[int, str]) -> Tuple[PlayerState, float, bool, None]:
        state, reward, end, _ = super().step(action)
        if self.result.status["dungeon_level"] == 2:
            end = True
            reward += self.stair_reward
        return state, reward, end, None
