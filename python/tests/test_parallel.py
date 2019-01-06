"""test for ParallelRogueEnv"""
from rogue_gym.envs import StairRewardParallel, ParallelRogueEnv
from data import CMD_STR, CMD_STR3, CMD_STR4, \
    CMD_STR5, SEED1_DUNGEON, SEED1_DUNGEON2, SEED1_DUNGEON3

CONFIG_ST = {
    "width": 32,
    "height": 16,
    "seed": 5,
    "hide_dungeon": False,
    "dungeon": {
        "style": "rogue",
        "room_num_x": 2,
        "room_num_y": 2,
    },
    "enemies": {
        "enemies": [],
    },
}
CONFIG_NOENEM = {
    "seed": 1,
}
NUM_WOKRERS = 8


def test_configs() -> None:
    env = ParallelRogueEnv(config_dicts=[CONFIG_NOENEM] * NUM_WOKRERS)
    for res in env.states:
        assert res.dungeon == SEED1_DUNGEON
    step = [CMD_STR, CMD_STR5]
    for i in range(len(CMD_STR)):
        env.step(''.join([step[x % 2][i] for x in range(NUM_WOKRERS)]))
    for i, res in enumerate(env.states):
        if i % 2 == 0:
            assert res.dungeon == SEED1_DUNGEON2
        else:
            assert res.dungeon == SEED1_DUNGEON3


def test_seed() -> None:
    env = ParallelRogueEnv(config_dicts=[CONFIG_NOENEM] * NUM_WOKRERS)
    for s in env.states:
        assert s.dungeon == SEED1_DUNGEON
    env.seed(10)
    res = env.reset()
    for s in res:
        assert s.dungeon != SEED1_DUNGEON


def test_step_cyclic() -> None:
    env = ParallelRogueEnv(config_dicts=[CONFIG_NOENEM] * NUM_WOKRERS, max_steps=5)
    for i, c in enumerate(CMD_STR):
        states, _, dones, _ = env.step(c * NUM_WOKRERS)
        if i == 4:
            assert dones == [True] * NUM_WOKRERS
            for res in states:
                assert res.dungeon == SEED1_DUNGEON
        else:
            assert dones == [False] * NUM_WOKRERS


def test_stair_reward() -> None:
    env = StairRewardParallel(config_dicts=[CONFIG_ST] * NUM_WOKRERS, max_steps=30)
    for c in CMD_STR3:
        _, rewards, *_ = env.step(c * NUM_WOKRERS)
        for r in rewards:
            assert r >= 0.0
    assert rewards == [50.0] * NUM_WOKRERS
    for c in CMD_STR4:
        _, rewards, *_ = env.step(c * NUM_WOKRERS)
        for r in rewards:
            assert r >= 0.0
    assert rewards == [50.0] * NUM_WOKRERS
    rest = 30 - (len(CMD_STR3) + len(CMD_STR4))
    for _ in range(rest):
        _, rewards, *_ = env.step([0] * NUM_WOKRERS)
        for r in rewards:
            assert r >= 0.0
