"""test for FirstFloorEnv and config string"""
from rogue_gym.envs import FirstFloorEnv
from data import CMD_STR2, SEED1_DUNGEON_CLEAR
import unittest

CONFIG = {
    "seed": 1,
    "hide_dungeon": False,
    "enemies": {
        "enemies": [],
    },
}

def test_configs(self):
    env = FirstFloorEnv(config_dict=CONFIG, stair_reward=100.0)
    assert env.get_dungeon().__len__() == SEED1_DUNGEON_CLEAR.__len__()
    state, rewards, done, _ = env.step(CMD_STR2)
    assert done == True
    assert rewards == 102
    assert env.channels() == 17
    symbol_img = env.symbol_image(state)
    assert symbol_img.shape == (17, 24, 80)
    assert env.get_config() == CONFIG

