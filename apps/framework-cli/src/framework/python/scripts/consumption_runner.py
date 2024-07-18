import asyncio
import dataclasses
from dataclasses import dataclass
from datetime import datetime, timezone

from clickhouse_connect import get_client
from string import Formatter
from importlib import import_module
from clickhouse_connect.driver.client import Client
import argparse
import os
import sys
import json

from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs



parser = argparse.ArgumentParser(description='Run Consumption Server')
parser.add_argument('consumption_dir_path', type=str,
                    help='Path to the consumption directory')
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
consumption_dir_path = args.consumption_dir_path

sys.path.append(consumption_dir_path)


# TODO: move this to python moose lib
class EnhancedJSONEncoder(json.JSONEncoder):
    def default(self, o):
        if isinstance(o, datetime):
            if o.tzinfo is None:
                o = o.replace(tzinfo=timezone.utc)
            return o.isoformat()
        if dataclasses.is_dataclass(o):
            return dataclasses.asdict(o)
        return super().default(o)


class MooseClient:
    def __init__(self, ch_client: Client):
        self.ch_client = ch_client

    def query(self, input, variables):
        params = {}
        values = {}

        for i, (_, variable_name, _, _) in enumerate(Formatter().parse(input)):
            if variable_name:
                value = variables[variable_name]
                if isinstance(value, list) and len(value) == 1:
                    # handling passing the value of the query string dict directly to variables
                    value = value[0]

                t = 'String' if isinstance(value, str) else \
                    'Int64' if isinstance(value, int) else \
                    'Float64' if isinstance(value, float) else "String"  # unknown type

                params[variable_name] = f'{{p{i}: {t}}}'
                values[f'p{i}'] = value
        clickhouse_query = input.format_map(params)
       
        val = self.ch_client.query(clickhouse_query, values)

        return list(val.named_results())


def handler_with_client(ch_client):
    class SimpleHTTPRequestHandler(BaseHTTPRequestHandler):
        def do_GET(self):
            parsed_path = urlparse(self.path)
            module_name = parsed_path.path.lstrip('/')
            try:
                module = import_module(module_name)
                # get the flow definition
                print(module_name)
                
                query_params = parse_qs(parsed_path.query)

                response = module.run(ch_client, query_params)
                response_message = bytes(json.dumps(response, cls=EnhancedJSONEncoder), 'utf-8')
                self.send_response(200)
                self.end_headers()
                self.wfile.write(response_message)
            except Exception as e:
                self.send_response(500)
                self.end_headers()
                self.wfile.write(str(e).encode())
    return SimpleHTTPRequestHandler


class DependencyError(Exception):
    pass


def get_file_name(path):
    return os.path.splitext(os.path.basename(path))[0]


def walk_dir(dir, file_extension):
    file_list = []

    for root, dirs, files in os.walk(dir):
        for file in files:
            if file.endswith(file_extension):
                file_list.append(os.path.join(root, file))

    return file_list


def main():
    print(f"Connecting to Clickhouse at {interface}://{host}:{port}")

    ch_client = get_client(interface=interface, host=host,
                           port=port, database=db, username=user, password=password)
    moose_client = MooseClient(ch_client)

    server_address = ('', 4001)
    handler = handler_with_client(moose_client)

    httpd = HTTPServer(server_address, handler)
    print(f"Starting server on http://localhost:4001")
    httpd.serve_forever()


main()
