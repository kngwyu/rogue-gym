#!/bin/sh

set -ex

cargo build --verbose --all
cargo test --verbose --all

cd python
python setup.py install
ls /home/travis/virtualenv/python3.6.3/lib/python3.6/site-packages/rouge_gym-0.1.0-py3.6-linux-x86_64.egg/rogue_gym/ -la
python setup.py test
