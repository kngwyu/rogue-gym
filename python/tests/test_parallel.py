"""test for ParallelRogueEnv"""
from rogue_gym.envs import ImageSetting, ParallelRogueEnv
from data import CMD_STR2, SEED1_DUNGEON


CONFIG_NOENEM = {
    "seed": 1,
    "enemies": {
        "enemies": [],
    },
}
NUM_WOKRERS = 8


def test_configs() -> None:
    env = ParallelRogueEnv(config_dicts=[CONFIG_NOENEM] * NUM_WOKRERS)
    for res in env.states:
        print(res.dungeon)
        assert res.dungeon == SEED1_DUNGEON
    env.step([ord('h')] * NUM_WOKRERS)
