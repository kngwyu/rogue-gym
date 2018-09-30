"""test for FirstFloorEnv and config string"""
from rogue_gym.envs import FirstFloorEnv
from data import CMD_STR2, SEED1_DUNGEON_CLEAR
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
        env = FirstFloorEnv(config_dict=CONFIG, stair_reward=100.0)
        self.assertEqual(env.get_screen(), SEED1_DUNGEON_CLEAR)
        _, rewards, done, _ = env.step(CMD_STR2)
        self.assertTrue(done)
        self.assertEqual(rewards, 102)


if __name__ == "__main__":
    unittest.main()
