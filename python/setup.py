import sys
from setuptools import find_packages, setup
from setuptools_rust import RustExtension


PYTHON_MAJOR_VERSION = sys.version_info[0]

setup_requires = ['setuptools-rust>=0.6.0']
install_requires = ['numpy', 'gym']
test_requires = install_requires + ['pytest']

setup(
    name='rouge-gym',
    version='0.1.0',
    description='OpenAI gym environment for rogue-gym',
    url='https://github.com/kngwyu/rogue-gym',
    author='Yuji Kanagawa',
    author_email='yuji.kngw.80s.revive@gmail.com',
    classifiers=[
        'License :: OSI Approved :: MIT License',
        'License :: OSI Approved :: Apache Software License',
        'Development Status :: 3 - Alpha',
        'Intended Audience :: Developers',
        'Programming Language :: Python',
        'Programming Language :: Rust',
        'Operating System :: POSIX',
    ],
    packages=find_packages(),
    rust_extensions=[RustExtension('rogue_gym_python._rogue_gym', 'Cargo.toml')],
    install_requires=install_requires,
    test_requires=test_requires,
    setup_requires=setup_requires,
    include_package_data=True,
    zip_safe=False,
)
