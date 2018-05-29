import sys
import gym
import numpy as np

try:
    from ._rogue_gym import sum_as_str
except ImportErroor as e:
    raise error.DependencyNotInstalled("{}. (Did you install cargo and rust?)")

__all__ = ['sum_as_str']
