# Developper notes

## Build manylinux wheel
```
cd ../
docker run --rm -v $PWD:/io quay.io/pypa/manylinux1_x86_64 /io/build-wheels.sh
```
