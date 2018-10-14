"""test for FirstFloorEnv and config string"""
from rogue_gym.envs import FirstFloorEnv
from data import CMD_STR2, SEED1_DUNGEON_CLEAR
import unittest

CONFIG = {
    "seed": 1,
    "hide_dungeon": False,
    "enemies": {
        "builtin": {
            "typ": "Rogue",
            "include": []
        }
    },
}


class TestSeed1(unittest.TestCase):
    """ test class for fixed seed
    """
    def test_configs(self):
        env = FirstFloorEnv(config_dict=CONFIG, stair_reward=100.0)
        self.assertEqual(env.get_dungeon().__len__(), SEED1_DUNGEON_CLEAR.__len__())
        state, rewards, done, _ = env.step(CMD_STR2)
        self.assertTrue(done)
        self.assertEqual(rewards, 102)
        self.assertEqual(env.channels(), 17)
        symbol_img = env.symbol_image(state)
        self.assertEqual(symbol_img.shape, (17, 24, 80))
        self.assertEqual(env.get_config(), CONFIG)
        symbol_img_hist = env.symbol_image_with_hist(state)
        self.assertEqual(symbol_img_hist.shape, (18, 24, 80))
        hist = symbol_img_hist[17]
        print(env.get_dungeon())
        print(hist[20])
        self.assertEqual(hist[20][10], 1.)
        gray_img = env.gray_image(state)
        self.assertEqual(gray_img.shape(), (1, 24, 80))
        gray_img_hist = env.gray_image_with_hist(state)
        self.assertEqual(gray_img_hist.shape(), (2, 24, 80))


if __name__ == "__main__":
    unittest.main()
