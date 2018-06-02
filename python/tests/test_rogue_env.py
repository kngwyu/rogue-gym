"""test for RogueEnv """
from rogue_gym import RogueEnv
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



class TestSeed1(unittest.TestCase):
    """ test class for fixed seed
    """
    
    def test_screen(self):
        env = RogueEnv()
        env.seed(1)
        env.reset()
        self.assertEqual(env.get_screen(), SEED1_DUNGEON)

    def test_action(self):
        env = RogueEnv()
        env.step("")
        self.assertEqual(env.get_screen(), SEED1_DUNGEON)

if __name__ == "__main__":
    unittest.main()
