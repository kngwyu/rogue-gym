from .rogue_env import PlayerState, RogueEnv
from .parallel import ParallelRogueEnv
from gym import Env, Wrapper
from typing import Iterable, List, Tuple, Union


def check_rogue_env(env: Env) -> None:
    if not isinstance(env.unwrapped, RogueEnv):
        raise ValueError('env have to be a wrapper of RoguEnv')


class StairRewardEnv(Wrapper):
    def __init__(self, env: Env, stair_reward: float = 50.0) -> None:
        check_rogue_env(env)
        self.stair_reward = stair_reward
        self.current_level = 1
        super().__init__(env)

    def step(self, action: Union[int, str]) -> Tuple[PlayerState, float, bool, None]:
        state, reward, end, info = self.env.step(action)
        current = self.unwrapped.result.status['dungeon_level']
        if self.current_level < current:
            self.current_level = current
            reward += self.stair_reward
        return state, reward, end, info

    def reset(self)  -> PlayerState:
        self.current_level = 1
        return super().reset()

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


class StairRewardParallel(ParallelRogueEnv):
    def __init__(self, *args, **kwargs) -> None:
        self.stair_reward = 50.0  # default reward
        if 'stair_reward' in kwargs:
            self.stair_reward = kwargs['stair_reward']
            del kwargs['stair_reward']
        super().__init__(*args, **kwargs)
        self.current_levels = [1] * self.num_workers

    def step(
            self,
            action: Union[Iterable[int], str]
    ) -> Tuple[List[PlayerState], List[float], List[bool], List[dict]]:
        state, reward, end, info = super().step(action)
        for i in range(self.num_workers):
            level = state[i].status['dungeon_level']
            if self.current_levels[i] < level:
                reward[i] += self.stair_reward
            self.current_levels[i] = level
        return state, reward, end, info
