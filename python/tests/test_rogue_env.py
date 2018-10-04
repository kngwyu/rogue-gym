"""test for RogueEnv """
from rogue_gym.envs import RogueEnv
from data import CMD_STR, SEED1_DUNGEON, SEED1_DUNGEON2
import unittest


class TestSeed1(unittest.TestCase):
    """ test class for fixed seed
    """
    def test_screen(self):
        env = RogueEnv(seed=1)
        self.assertEqual(env.get_dungeon(), SEED1_DUNGEON)
        h, w = env.screen_size()
        self.assertEqual(h, 24)
        self.assertEqual(w, 80)

    def test_action(self):
        env = RogueEnv(seed=1)
        res, _, _, _ = env.step(CMD_STR)
        self.assertEqual(res.dungeon, SEED1_DUNGEON2)


if __name__ == "__main__":
    unittest.main()
