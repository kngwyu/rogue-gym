#!/bin/sh

set -ex
echo $LIBRARY_PATH
echo $LD_LIBRARY_PATH

cargo build --verbose --all
cargo test --verbose --all

cd python
python setup.py install
python setup.py test
