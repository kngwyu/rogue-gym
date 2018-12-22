"""test for RogueEnv """
from data import CMD_STR, SEED1_DUNGEON, SEED1_DUNGEON2
import gym
from gym import spaces
import numpy as np
from rogue_gym.envs import StatusFlag, RogueEnv

CONFIG_NOENEM = {
    "seed": 1,
    "enemies": {
        "enemies": [],
    },
}


def test_screen():
    env = RogueEnv(seed=1)
    assert env.get_dungeon() == SEED1_DUNGEON
    h, w = env.screen_size()
    assert h == 24
    assert w == 80


def test_action():
    env = RogueEnv(seed=1)
    res, *_ = env.step(CMD_STR)
    assert res.dungeon == SEED1_DUNGEON2


def test_max_steps():
    env = RogueEnv(seed=1, max_steps=5)
    _, _, done, _ = env.step(CMD_STR)
    assert done


def test_images():
    env = RogueEnv(config_dict=CONFIG_NOENEM)
    state, *_ = env.step('H')
    status = StatusFlag.EMPTY
    symbol_img_hist = status.symbol_image_with_hist(state)
    assert symbol_img_hist.shape == (18, 24, 80)
    hist = symbol_img_hist[-1]
    for cell in hist[20][2:15]:
        assert cell, 1.
    gray_img = status.gray_image(state)
    assert gray_img.shape == (1, 24, 80)
    gray_img_hist = status.gray_image_with_hist(state)
    assert gray_img_hist.shape == (2, 24, 80)


def test_space():
    env = RogueEnv(config_dict=CONFIG_NOENEM)
    assert env.action_space == gym.spaces.discrete.Discrete(env.ACTION_LEN)
    # 26 = 17(symbols) + 9(all status)
    assert env.observation_space.shape == \
        spaces.box.Box(low=0, high=1, shape=(26, 24, 80), dtype=np.float32).shape
