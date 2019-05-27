#!/bin/bash

set -ex

cargo build --verbose --all
cargo test --manifest-path=core/Cargo.toml 
cd python
cargo test --no-default-features
tox -e py
