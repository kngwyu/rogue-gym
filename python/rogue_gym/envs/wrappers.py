from numpy import ndarray
from .rogue_env import RogueEnv, RogueResult
from typing import Tuple, Union


class FirstFloorEnv(RogueEnv):
    def step(self, action: Union[int, str]) -> Tuple[ndarray, float, bool, RogueResult]:
        features, reward, _, res = super().step(action)
        end = False
        if self.result.status["dungeon_level"] == 2:
            end = True
        return self.result.feature_map, reward, end, res
