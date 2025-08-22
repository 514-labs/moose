
from setuptools import setup

with open('requirements.txt') as f:
    requirements = [line.strip() for line in f if line.strip() and not line.startswith('#')]

setup(
    name='test_project',
    version='0.0',
    install_requires=requirements,
    python_requires='>=3.12',
)
