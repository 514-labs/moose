import asyncio
from dataclasses import dataclass
from clickhouse_connect import get_client
from importlib import import_module
import argparse
import os
import sys


parser = argparse.ArgumentParser(description='Run an aggregation')
parser.add_argument('aggregation_dir_path', type=str,
                    help='Path to the aggregation directory')
parser.add_argument('clickhouse_db', type=str, help='Clickhouse database name')
parser.add_argument('clickhouse_host', type=str, help='Clickhouse host')
parser.add_argument('clickhouse_port', type=int, help='Clickhouse port')
parser.add_argument('clickhouse_username', type=str,
                    help='Clickhouse username')
parser.add_argument('clickhouse_password', type=str,
                    help='Clickhouse password')
parser.add_argument('clickhouse_use_ssl', type=str, help='Clickhouse use SSL')


args = parser.parse_args()

interface = 'http' if args.clickhouse_use_ssl == "false" else 'https'
host = args.clickhouse_host
port = args.clickhouse_port
db = args.clickhouse_db
user = args.clickhouse_username
password = args.clickhouse_password
agg_dir_path = args.aggregation_dir_path

sys.path.append(agg_dir_path)


class DependencyError(Exception):
    pass


def get_file_name(path):
    return os.path.splitext(os.path.basename(path))[0]


def get_agg_from_file(path):
    # Make sure file path is in sys.path
    agg = import_module(get_file_name(path), package=agg_dir_path)

    # Find the Aggregation definition in the module
    agg_def = agg.Aggregration

    # Find the Aggregation objects in the module
    agg_objs = [obj for obj in dir(agg) if isinstance(
        getattr(agg, obj), agg_def)]

    # Make sure there is exactly one Aggregation object in the modules
    if len(agg_objs) == 0:
        raise ValueError(f"No Aggregation object found in {path}")

    if len(agg_objs) > 1:
        raise ValueError(f"Multiple Aggregation objects found in {path}")

    return getattr(agg, agg_objs[0])


def walk_dir(dir, file_extension):
    file_list = []

    for root, dirs, files in os.walk(dir):
        for file in files:
            if file.endswith(file_extension):
                file_list.append(os.path.join(root, file))

    return file_list


async def create_aggregation(ch_client, path):
    file_name = get_file_name(path)

    mv_obj = get_agg_from_file(path)

    mv_query = f"""
CREATE MATERIALIZED VIEW IF NOT EXISTS {file_name}
ENGINE = AggregatingMergeTree() ORDER BY {mv_obj.orderBy}
POPULATE
AS {mv_obj.select}
    """

    try:
        ch_client.command(mv_query)
        print(f"Created aggregation {file_name}. Query: {mv_query}")
    except Exception as err:
        print(f"Failed to create aggregation {file_name}: {err}")

        if 'UNKNOWN_TABLE' in str(err):
            raise DependencyError(str(err))


async def delete_aggregation(ch_client, path):
    file_name = get_file_name(path)

    try:
        ch_client.command(f"DROP VIEW IF EXISTS {file_name}")
        print(f"Deleted aggregation {file_name}")
    except Exception as err:
        print(f"Failed to delete aggregation {file_name}: {err}")


async def async_worker(task):
    await delete_aggregation(task['ch_client'], task['path'])
    await create_aggregation(task['ch_client'], task['path'])



async def main():
    print(f"Connecting to Clickhouse at {interface}://{host}:{port}")

    ch_client = get_client(interface=interface, host=host,
                           port=port, database=db, username=user, password=password)
    py_files = walk_dir(agg_dir_path, '.py')

    agg_files = []

    for file in py_files:
        try:
            get_agg_from_file(file)
            agg_files.append(file)
        except ValueError as err:
            print(f"Skipping {file}: {err}")

    print(f"Found {len(agg_files)} aggregations in {agg_dir_path}")
    print(f"Aggregations: {agg_files}")

    task_defs = [{'ch_client': ch_client, 'path': path, 'retries': len(agg_files)} for path in agg_files]

    print(f"Creating {len(task_defs)} tasks...")
    print(f"Tasks: {task_defs}")

    tasks = list(map(lambda x: asyncio.create_task(async_worker(x)), task_defs))

    print(f"Running {len(tasks)} tasks...")
    print(f"Tasks: {tasks}")

    while not all([task.done() for task in tasks]):
        await asyncio.sleep(1)

asyncio.run(main())
