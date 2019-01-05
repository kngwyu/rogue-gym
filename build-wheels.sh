#!/bin/bash
set -ex

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly  -y
export PATH="$HOME/.cargo/bin:$PATH"
cd /io/python

for PYBIN in /opt/python/cp{36,37}*/bin; do
    export PYTHON_SYS_EXECUTABLE="$PYBIN/python"
    "${PYBIN}/pip" install -U setuptools wheel==0.31.1 setuptools-rust
    "${PYBIN}/python" setup.py bdist_wheel
done

for whl in dist/*.whl; do
    auditwheel repair "$whl" -w dist/
done

