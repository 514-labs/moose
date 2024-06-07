import asyncio
from dataclasses import dataclass
from clickhouse_connect import Client as ClickhouseClient, get_client
import argparse

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
parser.add_argument('clickhouse_use_ssl', type=bool, help='Clickhouse use SSL')


args = parser.parse_args()

interface = 'http' if not args.clickhouse_use_ssl else 'https'
host = args.clickhouse_host
port = args.clickhouse_port
db = args.clickhouse_db
user = args.clickhouse_username
password = args.clickhouse_password

file_name = args.aggregation_dir_path.split('/')[-1]

client = get_client(interface=interface, host=host, port=port,
                    database=db, username=user, password=password)


@dataclass
class MvQuery:
    select: str
    order_by: str


@dataclass
class MvQueueTask:
    ch_client: ClickhouseClient
