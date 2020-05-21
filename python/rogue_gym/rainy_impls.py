from gym import Env
import numpy as np
from numpy import ndarray

try:
    from rainy.envs import EnvExt, EnvSpec, ParallelEnv
    from rainy.prelude import Array
except ImportError:
    raise ImportError("To use rogue_gym.rainy_impls, install rainy first.")
from .envs.parallel import ParallelRogueEnv
from .envs.rogue_env import PlayerState, RogueEnv
from .envs.wrappers import check_rogue_env
from typing import Iterable, Tuple

ACTION_DIM = len(RogueEnv.ACTIONS)


class RogueEnvExt(EnvExt):
    def __init__(self, env: Env) -> None:
        check_rogue_env(env)
        super().__init__(env)

    @property
    def action_dim(self) -> int:
        return ACTION_DIM

    @property
    def state_dim(self) -> Tuple[int, ...]:
        return self._env.unwrapped.observation_space.shape

    def extract(self, state: PlayerState) -> ndarray:
        return self._env.unwrapped.image_setting.expand(state)

    def save_history(self, file_name: str) -> None:
        self._env.unwrapped.save_actions(file_name)


class ParallelRogueEnvExt(ParallelEnv):
    def __init__(self, env: ParallelRogueEnv) -> None:
        self._env = env
        self._spec = EnvSpec(env.observation_space.shape, env.action_space)

    def close(self) -> None:
        self._env.close()

    def reset(self) -> Array[PlayerState]:
        return np.array(self._env.reset())

    def step(
        self, actions: Iterable[int]
    ) -> Tuple[Array[PlayerState], Array[float], Array[bool], Array[dict]]:
        return tuple(map(np.array, self._env.step(actions)))

    def seed(self, seeds: Iterable[int]) -> None:
        self._env.seed([s for s in seeds])

    @property
    def num_envs(self) -> int:
        return self.num_workers

    @property
    def spec(self) -> EnvSpec:
        return self._spec

    def extract(self, states: Iterable[PlayerState]) -> Array:
        return np.stack([self._env.image_setting.expand(state) for state in states])
