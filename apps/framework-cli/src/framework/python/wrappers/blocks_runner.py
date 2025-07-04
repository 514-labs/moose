import asyncio
from importlib import import_module
import argparse
import os
import sys
from moose_lib import cli_log, CliLogData


parser = argparse.ArgumentParser(description='Run blocks')
parser.add_argument('blocks_dir_path', type=str,
                    help='Path to the blocks directory')
parser.add_argument('clickhouse_db', type=str, help='Clickhouse database name')
parser.add_argument('clickhouse_host', type=str, help='Clickhouse host')
parser.add_argument('clickhouse_port', type=int, help='Clickhouse port')
parser.add_argument('clickhouse_username', type=str,
                    help='Clickhouse username')
parser.add_argument('clickhouse_password', type=str,
                    help='Clickhouse password')
parser.add_argument('clickhouse_use_ssl', type=str, help='Clickhouse use SSL')


args = parser.parse_args()

blocks_dir_path = args.blocks_dir_path

sys.path.append(blocks_dir_path)


def get_file_name(path):
    return os.path.splitext(os.path.basename(path))[0]


def get_blocks_from_file(path):
    # Make sure file path is in sys.path
    block = import_module(get_file_name(path), package=blocks_dir_path)

    # Find the Blocks definition in the module
    blocks_def = block.Blocks

    # Find the Blocks objects in the module
    blocks_objs = [obj for obj in dir(block) if isinstance(
        getattr(block, obj), blocks_def)]
    

    # Make sure there is exactly one Blocks object in the modules
    if len(blocks_objs) == 0:
        raise ValueError(f"No Blocks object found in {path}")

    if len(blocks_objs) > 1:
        raise ValueError(f"Multiple Blocks objects found in {path}")

    return getattr(block, blocks_objs[0])


def walk_dir(dir, file_extension):
    file_list = []

    for root, dirs, files in os.walk(dir):
        for file in files:
            if file.endswith(file_extension) and file != '__init__.py':
                file_list.append(os.path.join(root, file))

    return file_list


# DEPRECATED: These functions are no longer used since blocks functionality is deprecated
# They are kept for reference but will not be executed



async def main():
    py_files = walk_dir(blocks_dir_path, '.py')

    block_files = []

    for file in py_files:
        try:
            get_blocks_from_file(file)
            block_files.append(file)
        except Exception as err:
            # Ignore import errors for deprecation - we're not going to execute them anyway
            pass

    if len(block_files) > 0:
        print(f"⚠️  DEPRECATION WARNING: Blocks functionality has been deprecated and is no longer supported.")
        print(f"⚠️  Found {len(block_files)} blocks files in {blocks_dir_path}, but they will be ignored.")
        print(f"⚠️  Please migrate to the new data processing features. See documentation for alternatives.")
        
        cli_log(CliLogData(action="Blocks", message=f"Blocks functionality is deprecated. Found {len(block_files)} blocks files but ignoring them.", message_type="Warning"))
    else:
        print(f"No blocks files found in {blocks_dir_path}")

    # No-op: just return without doing anything
    return

asyncio.run(main())
