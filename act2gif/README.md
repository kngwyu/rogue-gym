# rogue-gym-act2gif
A simple tool to generate gif image from action history.

# usage
```
cargo run --release -- --actions='../data/learned/ddqn-minidungeon/best-actions.json' \
    --config='../data/learned/ddqn-minidungeon/config.json' --font=48 --interval=30 --max_actions=30
```
