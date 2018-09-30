from numpy import ndarray
from .rogue_env import RogueEnv, RogueResult
from typing import Tuple, Union


class FirstFloorEnv(RogueEnv):
    def __init__(
            self,
            seed: int = None,
            config_path: str = None,
            config_dict: dict = None,
            stair_reward: float = 100.0,
    ) -> None:
        super().__init__(seed=seed, config_path=config_path, config_dict=config_dict)
        self.stair_reward = stair_reward

    def step(self, action: Union[int, str]) -> Tuple[ndarray, float, bool, RogueResult]:
        features, reward, _, res = super().step(action)
        end = False
        if self.result.status["dungeon_level"] == 2:
            end = True
            reward += self.stair_reward
        return self.result.feature_map, reward, end, res
