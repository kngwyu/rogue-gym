import numpy as np
from numpy import ndarray
try:
    from rainy.envs import EnvExt, ParallelEnv
    from rainy.util.typehack import Array
except ImportError:
    raise ImportError('To use rogue_gym.rainy_impls, install rainy first.')
from .envs.parallel import ParallelRogueEnv
from .envs.rogue_env import PlayerState, RogueEnv
from typing import Iterable, Tuple
ACTION_DIM = len(RogueEnv.ACTIONS)


class RogueEnvExt(EnvExt, RogueEnv):
    @property
    def action_dim(self) -> int:
        return ACTION_DIM

    @property
    def state_dim(self) -> Tuple[int, ...]:
        return self.observation_space.shape

    def state_to_array(self, state: PlayerState) -> ndarray:
        return self.expand_state(state)

    def save_history(self, file_name: str) -> None:
        self.save_actions(file_name)


class ParallelRogueEnvExt(ParallelEnv, ParallelRogueEnv):
    def __init__(self, env: ParallelRogueEnv) -> None:
        self._env = env

    def close(self) -> None:
        pass

    def reset(self) -> Array[PlayerState]:
        return np.array(self._env.reset())

    def step(
            self,
            actions: Iterable[int]
    ) -> Tuple[Array[PlayerState], Array[float], Array[bool], Array[dict]]:
        return self._env.step(actions)

    def seed(self, seed: int) -> None:
        raise NotImplementedError('Please specify seed in config')

    def num_envs(self) -> int:
        return self.num_workers

    @property
    def action_dim(self) -> int:
        return ACTION_DIM

    @property
    def state_dim(self) -> Tuple[int, ...]:
        return self.observation_space.shape

    def states_to_array(self, states: Iterable[PlayerState]) -> Array:
        return np.stack([self._env.image_setting.expand(state) for state in states])
