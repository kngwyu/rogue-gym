from rogue_gym.envs import RogueEnv


def seed1():
    env = RogueEnv(seed=1)
    scr = env.get_screen()
    with open("../python/tests/data.py", "w") as data:
        data.write("SEED1_DUNGEON = [\n")
        for i, line in enumerate(scr):
            data.write("    ")
            data.write(str(line))
            if i == len(scr) - 1:
                data.write("]")
            else:
                data.write(",")
            data.write("\n")
        cmd_str = "jjjjjlllllllllj"
        scr, _ = env.step(cmd_str)
        data.write("CMD_STR = '" + cmd_str + "'\n")
        data.write("SEED1_DUNGEON2 = [\n")
        for i, line in enumerate(scr):
            data.write("    ")
            data.write(str(line))
            if i == len(scr) - 1:
                data.write("]")
            else:
                data.write(",")
            data.write("\n")

if __name__ == "__main__":
    seed1()
