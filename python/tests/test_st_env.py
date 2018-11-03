"""test for StairRewardEnv"""
from rogue_gym.envs import StairRewardEnv
from data import CMD_STR3, CMD_STR4
import unittest

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
        "builtin": {
            "typ": "Rogue",
            "include": []
        }
    }
}


class TestSeed1(unittest.TestCase):
    """ test class for fixed seed
    """
    def test_configs(self):
        env = StairRewardEnv(config_dict=CONFIG, stair_reward=100.0)
        state, rewards, done, _ = env.step(CMD_STR3)
        self.assertEqual(rewards, 104.0)
        state, rewards, _, _ = env.step(CMD_STR4)
        self.assertEqual(rewards, 100.0)
        img = env.symbol_image_with_hist_and_level(state)
        self.assertEqual(img.shape, (19, 16, 32))
        self.assertEqual(img[18][0][0], 3.0)


if __name__ == "__main__":
    unittest.main()
