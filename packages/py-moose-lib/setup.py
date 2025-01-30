import sys
from setuptools import setup, find_packages

version = "0.0.0"

if '--version' in sys.argv:
    index = sys.argv.index('--version')
    sys.argv.pop(index)
    version = sys.argv.pop(index)

setup(
    name='moose_lib',
    version=version,
    packages=find_packages(),
    install_requires=[
        'pyjwt[crypto]==2.9.0',
        'asyncio==3.4.3',
    ],
)