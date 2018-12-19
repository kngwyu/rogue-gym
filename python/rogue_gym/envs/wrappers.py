from .rogue_env import PlayerState, RogueEnv
from typing import Tuple, Union


class StairRewardEnv(RogueEnv):
    def __init__(self, *args, **kwargs) -> None:
        self.stair_reward = 50.0  # default reward
        if 'stair_reward' in kwargs:
            self.stair_reward = kwargs['stair_reward']
            del kwargs['stair_reward']
        super().__init__(*args, **kwargs)
        self.current_level = 1

    def step(self, action: Union[int, str]) -> Tuple[PlayerState, float, bool, None]:
        state, reward, end, info = super().step(action)
        current = self.result.status["dungeon_level"]
        if self.current_level < current:
            self.current_level = current
            reward += self.stair_reward
        return state, reward, end, info

    def __repr__(self):
        return super().__repr__()


class FirstFloorEnv(StairRewardEnv):
    def step(self, action: Union[int, str]) -> Tuple[PlayerState, float, bool, None]:
        state, reward, end, info = super().step(action)
        if self.current_level == 2:
            end = True
        return state, reward, end, info

    def __repr__(self):
        return super().__repr__()
