from gym.envs.registration import register

register(
    id = 'Rogue-v0',
    entry_point = 'rogue_gym.envs:RogueEnv',
    timestep_limit = 1000,
    reward_threshold = 1.0,
    nondeterministic = True,
)
