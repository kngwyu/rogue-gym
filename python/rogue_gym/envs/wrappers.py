from .rogue_env import PlayerState, RogueEnv
from typing import Tuple, Union


class FirstFloorEnv(RogueEnv):
    def __init__(
            self,
            seed: int = None,
            config_path: str = None,
            config_dict: dict = None,
            max_steps: int = 1000,
            stair_reward: float = 100.0,
    ) -> None:
        super().__init__(
            seed=seed,
            config_path=config_path,
            config_dict=config_dict,
            max_steps=max_steps
        )
        self.stair_reward = stair_reward

    def step(self, action: Union[int, str]) -> Tuple[PlayerState, float, bool, None]:
        state, reward, end, _ = super().step(action)
        if self.result.status["dungeon_level"] == 2:
            end = True
            reward += self.stair_reward
        return state, reward, end, None
