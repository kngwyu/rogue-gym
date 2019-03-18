# rogue-gym
[![Buid Status](https://travis-ci.org/kngwyu/rogue-gym.svg?branch=master)](https://travis-ci.org/kngwyu/rogue-gym)
[![PyPI version](https://img.shields.io/pypi/v/rogue_gym.svg)](https://pypi.org/project/rogue-gym/)

Highly customizable rogue-like implmentation for AI expmeriments.

# Play as human

```
git clone https://github.com/kngwyu/rogue-gym.git
cd rogue-gym/rogue-gym-devui
cargo run --release
```

# Watch learned AI
![Double DQN gif](data/gif/ddqn-small-16.gif)

Now this repository has Double DQN result.

```
cargo install --path ./devui --force
cd data/learned/ddqn-minidungeon/
rogue_gym_devui --config config.json replay --file best-actions.json --interval 100
```

# Python API

See [this page](./python/README.md)

# Acknowledgements
[rogue5.4](https://github.com/kngwyu/rogue5.4.4)

# Required minimum version of rust
- core/devui 1.31.0-beta
- python 1.32.0 nightly

# License

This project itself is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

This project contains some third party products.

The following third party products are included, and carry their
own copyright notices and license terms:


- [Ubuntu mono font](./data/fonts/UbuntuMono-R.ttf) is distributed
under [UBUNTU FONT LICENCE](./data/fonts/LICENCE.txt)
