"""test for FirstFloorEnv and config string"""
from rogue_gym.envs import FirstFloorEnv
from data import SEED1_DUNGEON_CLEAR
import unittest

CONFIG = {
    "dungeon-style": "rogue",
    "dungeon-setting": {},
    "seed": 1,
    "hide_dungeon": False
}


class TestSeed1(unittest.TestCase):
    """ test class for fixed seed
    """
    def test_screen(self):
        env = FirstFloorEnv(config_dict=CONFIG)
        env.show_screen()
        self.assertEqual(env.get_screen(), SEED1_DUNGEON_CLEAR)


if __name__ == "__main__":
    unittest.main()
