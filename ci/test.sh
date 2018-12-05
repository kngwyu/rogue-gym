#!/bin/sh

set -ex

cargo build --verbose --all
cargo test --verbose --all

cd python
tox -e py
