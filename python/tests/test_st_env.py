"""test for StairRewardEnv"""
from rogue_gym.envs import StairRewardEnv, ImageSetting, StatusFlag, DungeonType
from data import CMD_STR3, CMD_STR4

CONFIG = {
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

EXPAND = ImageSetting(
    DungeonType.SYMBOL,
    StatusFlag.DUNGEON_LEVEL | StatusFlag.HP_CURRENT | StatusFlag.EXP,
    True,
)


def test_configs():
    env = StairRewardEnv(config_dict=CONFIG, stair_reward=100.0, image_setting=EXPAND)
    state, rewards, done, _ = env.step(CMD_STR3)
    assert rewards == 104.0
    state, rewards, _, _ = env.step(CMD_STR4)
    assert rewards == 100.0
    img = env.state_to_image(state)
    assert img.shape == (21, 16, 32)
    assert img[17][0][0] == 3.0
    assert img[18][0][0] == 12.0
    assert StatusFlag.FULL.status_vec(state) == [3, 12, 12, 16, 16, 0, 1, 0, 0]
