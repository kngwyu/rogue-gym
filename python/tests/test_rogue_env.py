"""test for RogueEnv """
from rogue_gym.envs import BaseEnv
from data import *
import unittest

class TestSeed1(unittest.TestCase):
    """ test class for fixed seed
    """
    def test_screen(self):
        env = BaseEnv(seed=1)
        env.show_screen()
        self.assertEqual(env.get_screen(), SEED1_DUNGEON)

    def test_action(self):
        env = BaseEnv(seed=1)
        _, _, _, res = env.step(CMD_STR)
        self.assertEqual(res.dungeon, SEED1_DUNGEON2)

if __name__ == "__main__":
    unittest.main()
