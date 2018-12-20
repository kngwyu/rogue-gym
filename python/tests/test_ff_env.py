"""test for FirstFloorEnv and config string"""
from rogue_gym.envs import ImageSetting, FirstFloorEnv, StatusFlag
from data import CMD_STR2, SEED1_DUNGEON_CLEAR

CONFIG = {
    "seed": 1,
    "hide_dungeon": False,
    "enemies": {
        "enemies": [],
    },
}

EXPAND = ImageSetting(status=StatusFlag.DUNGEON_LEVEL)


def test_configs():
    env = FirstFloorEnv(config_dict=CONFIG, stair_reward=100.0, image_setting=EXPAND)
    assert env.get_dungeon().__len__() == SEED1_DUNGEON_CLEAR.__len__()
    state, rewards, done, _ = env.step(CMD_STR2)
    assert done
    assert rewards == 102
    symbol_img = env.state_to_image(state)
    assert symbol_img.shape == (18, 24, 80)
    assert env.get_config() == CONFIG
