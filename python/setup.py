import io
import os
import re
from setuptools import find_packages, setup
from setuptools_rust import RustExtension


NAME = "rogue-gym"
AUTHOR = "Yuji Kanagawa"
EMAIL = "yuji.kngw.80s.revive@gmail.com"
URL = "https://github.com/kngwyu/rogue-gym"
REQUIRES_PYTHON = ">=3.6.0"
DESCRIPTION = "OpenAI gym environment for rogue-gym"


here = os.path.abspath(os.path.dirname(__file__))
with io.open(os.path.join(here, "rogue_gym/__init__.py"), encoding="utf-8") as f:
    VERSION = re.search(r"__version__ = \'(.*?)\'", f.read()).group(1)

try:
    with io.open(os.path.join(here, "README.md"), encoding="utf-8") as f:
        LONG_DESCRIPTION = "\n" + f.read()
except FileNotFoundError:
    LONG_DESCRIPTION = DESCRIPTION


SETUP = ["setuptools-rust>=0.6.0"]
REQUIRED = ["numpy", "gym"]
TEST = ["pytest"]
EXTRA = {"rainy": ["rainy"]}

setup(
    name=NAME,
    version=VERSION,
    url=URL,
    project_urls={"Code": URL, "Issue tracker": URL + "/issues",},
    description=DESCRIPTION,
    long_description=LONG_DESCRIPTION,
    long_description_content_type="text/markdown",
    author=AUTHOR,
    author_email=EMAIL,
    classifiers=[
        "License :: OSI Approved :: MIT License",
        "License :: OSI Approved :: Apache Software License",
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "Natural Language :: English",
        "Operating System :: Microsoft :: Windows",
        "Operating System :: POSIX :: Linux",
        "Operating System :: MacOS",
        "Programming Language :: Rust",
        "Programming Language :: Python :: 3.6",
        "Programming Language :: Python :: 3.7",
    ],
    license="MIT",
    packages=find_packages(),
    rust_extensions=[RustExtension("rogue_gym_python._rogue_gym", "Cargo.toml")],
    setup_requires=SETUP,
    tests_require=TEST,
    install_requires=REQUIRED,
    extras_require=EXTRA,
    include_package_data=True,
    zip_safe=False,
)
