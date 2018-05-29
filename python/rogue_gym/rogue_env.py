class RogueEnv(gym.Env):
    metadata = {'render.modes': ['human', 'ansi']}
    def __init__(self):
        super().__init__()
