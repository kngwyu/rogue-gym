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


if __name__ == "__main__":
    unittest.main()
