import argparse
from .envs import RogueEnv


def play_cli() -> None:
    parser = argparse.ArgumentParser(description='Play rogue-gym CLI')
    parser.add_argument(
        '--config',
        metavar='-C',
        type=str,
        nargs=1,
        default=None,
        help='path to config json file'
    )
    args = parser.parse_args()
    env = RogueEnv(config_path=args.config)
    env.play_cli()


if __name__ == '__main__':
    play_cli()
