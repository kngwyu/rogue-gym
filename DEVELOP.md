# Developper notes

## Build manylinux wheel
```
docker run --rm -v $PWD:/io quay.io/pypa/manylinux1_x86_64 /io/build-wheels.sh
```
