
from setuptools import setup

setup(
    name='test_project',
    version='0.0',
    install_requires=[
        "kafka-python-ng==2.2.2",
        "clickhouse_connect==0.7.16",
        "temporalio==1.9.0",
        "requests==2.32.3",
        "moose-cli",
        "moose-lib",
    ],
)
