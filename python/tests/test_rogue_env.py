"""test for RogueEnv """
from rogue_gym.envs import RogueEnv
from data import CMD_STR, SEED1_DUNGEON, SEED1_DUNGEON2
import unittest

CONFIG_NOENEM = {
    "seed": 1,
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

    def test_max_steps(self):
        env = RogueEnv(seed=1, max_steps=5)
        _, _, done, _ = env.step(CMD_STR)
        self.assertTrue(done)

    def test_images(self):
        env = RogueEnv(config_dict=CONFIG_NOENEM)
        state, _, _, _ = env.step('H')
        symbol_img_hist = env.symbol_image_with_hist(state)
        self.assertEqual(symbol_img_hist.shape, (18, 24, 80))
        hist = symbol_img_hist[-1]
        for cell in hist[20][2:15]:
            self.assertEqual(cell, 1.)
        gray_img = env.gray_image(state)
        self.assertEqual(gray_img.shape, (1, 24, 80))
        gray_img_hist = env.gray_image_with_hist(state)
        self.assertEqual(gray_img_hist.shape, (2, 24, 80))


if __name__ == "__main__":
    unittest.main()
