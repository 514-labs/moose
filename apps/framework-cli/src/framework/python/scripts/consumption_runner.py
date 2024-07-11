import asyncio
from dataclasses import dataclass
from clickhouse_connect import get_client
from string import Formatter
from importlib import import_module
from clickhouse_connect.driver.client import Client
import argparse
import os
import sys
import json
import datetime

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

class DateTimeEncoder(json.JSONEncoder):
    def default(self, obj):
        if isinstance(obj, datetime.datetime):
            return obj.isoformat()
        return super().default(obj)

class MooseClient:
    def __init__(self, ch_client):
        self.ch_client = ch_client

    def query(self, input, variables):
        fieldnames = [fname for _, fname, _, _ in Formatter().parse(input) if fname]

        # Create a dictionary with keys as fieldnames and values as the corresponding string format
        params = {fname: f'{{p{i}: String}}' for i, fname in enumerate(fieldnames)}

        values = {f'p{i}': variables[fname][0] if isinstance(variables[fname], list) and len(variables[fname]) == 1 else variables[fname] for i, fname in enumerate(fieldnames) if fname in variables}
        clickhouse_query = input.format_map(params)
       
        val = self.ch_client.query(clickhouse_query, values)
        return val.result_rows

def handler_with_client(ch_client):
    class SimpleHTTPRequestHandler(BaseHTTPRequestHandler):
        def do_GET(self):
            parsed_path = urlparse(self.path)
            module_name = parsed_path.path.lstrip('/')
            if module_name.startswith('consumption/'):
                module_name = module_name[len('consumption/'):]
            try:
                module = import_module(module_name)

                # get the flow definition

                print(module_name)
                
                query_params = parse_qs(parsed_path.query)


                response = module.run(ch_client, query_params)
                response_message = bytes(json.dumps(response.message, cls=DateTimeEncoder), 'utf-8')
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

async def main():
    print(f"Connecting to Clickhouse at {interface}://{host}:{port}")

    ch_client = get_client(interface=interface, host=host,
                           port=port, database=db, username=user, password=password)
    moose_client = MooseClient(ch_client)

    server_address = ('', 4001)
    handler = handler_with_client(moose_client)

    httpd = HTTPServer(server_address, handler)
    print(f"Starting server on http://localhost:4001")
    httpd.serve_forever()






asyncio.run(main())