# rogue-gym
[![Buid Status](https://travis-ci.org/kngwyu/rogue-gym.svg?branch=master)](https://travis-ci.org/kngwyu/rogue-gym)

Highly customizable rogue-like implmentation for AI expmeriments.

# Play as human

```
git clone https://github.com/kngwyu/rogue-gym.git
cd rogue-gym/rogue-gym-devui
cargo run --release
```

# Watch learned AI
Now this repository has Double DQN result
```
cargo install --path ./devui --force
cd data/learned/ddqn-minidugeon/
rogue_gym_devui --config config.json replay --file best-actions.json --interval 100
```

# Python API

See [this page](./python/README.md)

# Acknowledgements
[rogue5.4](https://github.com/kngwyu/rogue5.4.4)

# Required minimum version of rust
1.31.0-beta

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.
