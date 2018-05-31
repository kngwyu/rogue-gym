"""test for RogueEnv """
from rogue_gym import RogueEnv
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

def test_seed1():
    env = RogueEnv()
    env.seed(1)
    env.reset()
    assert env.get_screen(), SEED1_DUNGEON

def test():
    test_seed1()

if __name__ == "__main__":
    test()
