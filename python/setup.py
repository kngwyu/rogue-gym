import sys
from setuptools import find_packages, setup
from setuptools_rust import RustExtension


PYTHON_MAJOR_VERSION = sys.version_info[0]

setup_requirements = ['setuptools-rust>=0.6.0']
install_requirements = ['numpy', 'gym']
test_requirements = install_requirements + ['pytest']
extra_requirements = {'rainy': ['rainy']}

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
    tests_require=test_requirements,
    install_requires=install_requirements,
    setup_requires=setup_requirements,
    extras_require=extra_requirements,
    include_package_data=True,
    zip_safe=False,
)
