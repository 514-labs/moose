import argparse
import dataclasses
import json
import os
import subprocess
import sys
import traceback
from datetime import datetime, timezone, date

from http import HTTPStatus
from http.server import HTTPServer, BaseHTTPRequestHandler

from importlib import import_module
from string import Formatter
from typing import Optional, Dict, Any
from urllib.parse import urlparse, parse_qs
from moose_lib.query_param import QueryField, ArrayType, convert_consumption_api_param, parse_scalar_value

import jwt
from clickhouse_connect import get_client
from clickhouse_connect.driver.client import Client

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
parser.add_argument('jwt_secret', type=str, help='JWT secret')
parser.add_argument('jwt_issuer', type=str, help='JWT issuer')
parser.add_argument('jwt_audience', type=str, help='JWT audience')
parser.add_argument('jwt_enforce_all', type=str, help='Auto-handle requests without JWT')

args = parser.parse_args()

interface = 'http' if args.clickhouse_use_ssl == "false" else 'https'
host = args.clickhouse_host
port = args.clickhouse_port
db = args.clickhouse_db
user = args.clickhouse_username
password = args.clickhouse_password
consumption_dir_path = args.consumption_dir_path

jwt_secret = args.jwt_secret
jwt_issuer = args.jwt_issuer
jwt_audience = args.jwt_audience
jwt_enforce_all = args.jwt_enforce_all

sys.path.append(consumption_dir_path)


def map_params_to_dataclass(params:  dict[str, list[str]], fields: list[QueryField], cls: type) -> Any:
    # Initialize an empty dict for the constructor arguments
    constructor_args = {}

    # Get field definitions from the dataclass
    for field_def in fields:
        field_name = field_def.name

        # Skip if field not in params
        if field_name not in params:
            continue

        # Get the value(s) from the params list
        values = params[field_name]
        if not values:
            continue

        # Get the field type, handling Optional types
        field_type = field_def.data_type

        if isinstance(field_type, ArrayType):
            constructor_args[field_name] = [parse_scalar_value(v, field_type.element_type) for v in values]

        if len(values) != 1:
            raise ValueError(f"Expected a single element for {field_name}")
        [v] = values
        constructor_args[field_name] = parse_scalar_value(v, field_type)

    # Create and return instance of the dataclass
    return cls(**constructor_args)



# TODO: move this to python moose lib
class EnhancedJSONEncoder(json.JSONEncoder):
    def default(self, o):
        if isinstance(o, datetime):
            if o.tzinfo is None:
                o = o.replace(tzinfo=timezone.utc)
            return o.isoformat()
        if isinstance(o, date):
            return o.isoformat()
        if dataclasses.is_dataclass(o):
            return dataclasses.asdict(o)
        return super().default(o)


class QueryClient:
    def __init__(self, ch_client: Client):
        self.ch_client = ch_client

    def execute(self, input, variables):
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

class WorkflowClient:
    def execute(self, name: str, input_data: Any) -> None:
        moose_cli = "/usr/local/bin/moose-cli"

        try:
            version_command = [moose_cli, "--version"]
            process = subprocess.Popen(version_command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            stdout, stderr = process.communicate()

            if process.returncode != 0:
                print(f"WorkflowClient - failed to get moose-cli version: {stderr.strip()}")
                return

            print(f"WorkflowClient - moose-cli version: {stdout.strip()}")
        except FileNotFoundError:
            print("WorkflowClient - moose-cli not found. Please ensure it is installed and the path is correct.")
            print(f"WorkflowClient - current PATH: {os.environ.get('PATH')}")
            return
        except Exception as e:
            print(f"WorkflowClient - an error occurred while checking moose-cli version: {str(e)}")
            print(f"WorkflowClient - current PATH: {os.environ.get('PATH')}")
            return
        
        try:
            input_data_json = json.dumps(input_data, cls=EnhancedJSONEncoder)
            workflow_command = [moose_cli, "workflow", "run", name, "--input", input_data_json]
            print(f"WorkflowClient - executing command: {workflow_command} with input: {input_data_json}")

            try:
                process = subprocess.Popen(workflow_command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                stdout, stderr = process.communicate()
                if stdout:
                    print("WorkflowClient - command output:", stdout)
                    return {
                        "status": 200,
                        "body": stdout
                    }
                if stderr:
                    print("WorkflowClient - command error:", stderr)
                    return {
                        "status": 400,
                        "body": stderr
                    }
            except Exception as e:
                print(f"WorkflowClient - an error occurred while running the command: {e}")
        except (TypeError, ValueError) as e:
            print(f"WorkflowClient - failed to convert input data to JSON: {e}")

class MooseClient:
    def __init__(self, ch_client: Client):
        self.query = QueryClient(ch_client)
        self.workflow = WorkflowClient()

def verify_jwt(token: str) -> Optional[Dict[str, Any]]:
    try:
        payload = jwt.decode(token, jwt_secret, algorithms=["RS256"], audience=jwt_audience, issuer=jwt_issuer)
        return payload
    except Exception as e:
        print("JWT verification failed:", str(e))
        return None

def has_jwt_config() -> bool:
    return jwt_secret and jwt_issuer and jwt_audience

def handler_with_client(ch_client):
    class SimpleHTTPRequestHandler(BaseHTTPRequestHandler):
        def log_request(self, code = "-", size = "-"):
            """instead of calling log_message which goes to stderr by default,
            this implementation goes to stdout, but is otherwise the same.
            """
            if isinstance(code, HTTPStatus):
                code = code.value
            sys.stdout.write('%s - - [%s] "%s" %s %s\n' %
                             (self.address_string(),
                              self.log_date_time_string(),
                              self.requestline,
                              str(code),
                              str(size)))
        def do_GET(self):
            parsed_path = urlparse(self.path)
            module_name = parsed_path.path.lstrip('/')
            try:
                module = import_module(module_name)
                fields_and_class = convert_consumption_api_param(module)

                if has_jwt_config():
                    jwt_payload: Optional[Dict[str, Any]] = None
                    auth_header = self.headers.get('Authorization')
                    if auth_header:
                        # Bearer <token>
                        token = auth_header.split(" ")[1] if " " in auth_header else None
                        if token:
                            jwt_payload = verify_jwt(token)
                    
                    if jwt_payload is None and jwt_enforce_all == 'true':
                        self.send_response(401)
                        response_message = bytes(json.dumps({"error": "Unauthorized"}), 'utf-8')
                    else:
                        query_params = parse_qs(parsed_path.query)
                        if fields_and_class is not None:
                            (cls, fields) = fields_and_class
                            query_params = map_params_to_dataclass(query_params, fields, cls)
                        response = module.run(ch_client, query_params, jwt_payload)
                        if hasattr(response, 'status') and hasattr(response, 'body'):
                            response_message = bytes(json.dumps(response.body, cls=EnhancedJSONEncoder), 'utf-8')
                            self.send_response(response.status)
                        else:
                            response_message = bytes(json.dumps(response, cls=EnhancedJSONEncoder), 'utf-8')
                            self.send_response(200)

                else:
                    query_params = parse_qs(parsed_path.query)
                    if fields_and_class is not None:
                        (cls, fields) = fields_and_class
                        query_params = map_params_to_dataclass(query_params, fields, cls)
                    response = module.run(ch_client, query_params)
                    if hasattr(response, 'status') and hasattr(response, 'body'):
                        response_message = bytes(json.dumps(response.body, cls=EnhancedJSONEncoder), 'utf-8')
                        self.send_response(response.status)
                    else:
                        response_message = bytes(json.dumps(response, cls=EnhancedJSONEncoder), 'utf-8')
                        self.send_response(200)

                self.end_headers()
                self.wfile.write(response_message)

            except Exception as e:
                traceback.print_exc()
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
