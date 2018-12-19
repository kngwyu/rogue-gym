from numpy import ndarray
try:
    from rainy.envs import EnvExt
except ImportError:
    raise ImportError('To use rogue_gym.rainy_impls, install rainy first.')
from .rogue_env import PlayerState, RogueEnv
from typing import Tuple
ACTION_DIM = len(RogueEnv.ACTIONS)


class RogueEnvExt(EnvExt):
    def __init__(self, env: RogueEnv) -> None:
        super().__init__(env)

    @property
    def action_dim(self) -> int:
        return ACTION_DIM

    @property
    def state_dim(self) -> Tuple[int, ...]:
        return self._env.observation_space.shape

    def state_to_array(self, state: PlayerState) -> ndarray:
        return self._env.expand_state(state)

    def save_history(self, file_name: str) -> None:
        self._env.save_actions(file_name)
