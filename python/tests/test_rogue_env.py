"""test for RogueEnv """
import rogue_gym
import unittest
SEED1_DUNGEON = [b'                                                                                ',
                 b'                                                    ---------                   ',
                 b'                                                    +....*..|                   ',
                 b'                                                    |.@.....|                   ',
                 b'                                                    ---------                   ',
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
                 b'                                                                                ',
                 b'                                                                                ',
                 b'                                                                                ']

class TestSeed1(unittest.TestCase):
    """ test class for fixed seed
    """
    
    def test_screen(self):
        env = rogue_gym.RogueEnv()
        env.seed(1)
        env.reset()
        self.assertEqual(env.get_screen(), SEED1_DUNGEON)

if __name__ == "__main__":
    print(dir(rogue_gym))
    print(rogue_gym.__path__)
    unittest.main()
