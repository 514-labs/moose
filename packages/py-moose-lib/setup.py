import sys
from setuptools import setup, find_packages

from pathlib import Path

this_directory = Path(__file__).parent
long_description = (this_directory / "README.md").read_text()

version = "0.0.0"

if '--version' in sys.argv:
    index = sys.argv.index('--version')
    sys.argv.pop(index)
    version = sys.argv.pop(index)

setup(
    name='moose_lib',
    version=version,
    packages=find_packages(),
    long_description=long_description,
    long_description_content_type='text/markdown',
    author='Fiveonefour Labs Inc.',
    author_email='support@fiveonefour.com',
    url='https://www.fiveonefour.com/moose',
    install_requires=[
        'pyjwt[crypto]==2.9.0',
        'asyncio==3.4.3',
        "pydantic==2.10.6",
        "temporalio==1.9.0",
        "kafka-python-ng==2.2.2",
    ],
)
