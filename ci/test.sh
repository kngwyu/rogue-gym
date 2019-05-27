#!/bin/sh

set -ex

cargo build --verbose --all
cd core
cargo test --verbose
cd ../python
cargo test --no-default-features
tox -e py
