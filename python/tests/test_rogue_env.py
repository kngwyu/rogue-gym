"""test for RogueEnv """
from rogue_gym.envs import RogueEnv
import unittest
SEED1_DUNGEON = [
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'--------------------                                                            ',
    b'|..*...............+                                                            ',
    b'|..............@...|                                                            ',
    b'--------------------                                                            ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ']

SEED1_DUNGEON2 = [
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'--------------------                                                            ',
    b'|..@...............+                                                            ',
    b'|..................|                                                            ',
    b'--------------------                                                            ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ',
    b'                                                                                ']

class TestSeed1(unittest.TestCase):
    """ test class for fixed seed
    """
    
    def test_screen(self):
        env = RogueEnv(seed = 1)
        self.assertEqual(env.get_screen(), SEED1_DUNGEON)

    def test_action(self):
        env = RogueEnv(seed = 1)
        screen, status = env.step("hhhhhhhhhhhhk")
        self.assertEqual(screen, SEED1_DUNGEON2)
        self.assertEqual(status["gold"], 2)

if __name__ == "__main__":
    unittest.main()
