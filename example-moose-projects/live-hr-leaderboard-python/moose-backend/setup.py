
from setuptools import setup

setup(
    name='live-hr-leaderboard-python',
    version='0.0',
    install_requires=[
        "kafka-python-ng==2.2.2",
        "clickhouse_connect==0.7.16",
        "temporalio==1.9.0",
        "requests==2.32.3",
        "moose-cli==0.3.805", # Fixing version
        "moose-lib==0.3.805", # Fixing version
        "bleak==0.22.3",
    ],
)
