import argparse
import asyncio
import dataclasses
import json
import os
import subprocess
import sys
import traceback
from datetime import datetime, timezone, date, timedelta

from http import HTTPStatus
from http.server import HTTPServer, BaseHTTPRequestHandler

from importlib import import_module
from string import Formatter
from typing import Optional, Dict, Any
from urllib.parse import urlparse, parse_qs
from moose_lib.query_param import map_params_to_class, convert_consumption_api_param

import jwt
from clickhouse_connect import get_client
from clickhouse_connect.driver.client import Client as ClickhouseClient

from temporalio.client import Client as TemporalClient
from temporalio.common import RetryPolicy


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
    def __init__(self, ch_client: ClickhouseClient):
        self.ch_client = ch_client

    def __call__(self, input, variables):
        return self.execute(input, variables)

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
       
        # We are not using the result of the ping
        # but this ensures that if the clickhouse cloud service is idle, we 
        # wake it up, before we send the query.
        self.ch_client.ping()

        val = self.ch_client.query(clickhouse_query, values)

        return list(val.named_results())

class WorkflowClient:
    def __init__(self, temporal_client: TemporalClient):
        self.temporal_client = temporal_client
        self.configs = self.load_consolidated_configs()
        print(f"WorkflowClient - configs: {self.configs}")

    # Test workflow executor in rust if this changes significantly
    def execute(self, name: str, input_data: Any) -> Dict[str, Any]:
        try:
            run_id = asyncio.run(self._start_workflow_async(name, input_data))
            print(f"WorkflowClient - started workflow: {name}")
            return {
                "status": 200,
                "body": f"Workflow started: {name}. View it in the Temporal dashboard: http://localhost:8080/namespaces/default/workflows/{name}/{run_id}/history"
            }
        except Exception as e:
            print(f"WorkflowClient - error while starting workflow: {e}")
            return {
                "status": 400,
                "body": str(e)
            }
    
    async def _start_workflow_async(self, name: str, input_data: Any):
        if 'retries' not in self.configs.get(name, {}):
            raise ValueError(f"Missing 'retries' configuration for workflow: {name}")
        retry_count = self.configs[name]['retries']
        retry_policy = RetryPolicy(
            maximum_attempts=retry_count
        )

        if 'timeout' not in self.configs.get(name, {}):
            raise ValueError(f"Missing 'timeout' configuration for workflow: {name}")
        timeout_str = self.configs[name]['timeout']
        run_timeout = self.parse_timeout_to_timedelta(timeout_str)

        print(f"WorkflowClient - starting workflow: {name} with retry policy: {retry_policy} and timeout: {run_timeout}")
        
        # We should parse and encode the input_data here
        if input_data:
            try:
                # First decode the JSON string if it's a string
                if isinstance(input_data, str):
                    input_data = json.loads(input_data)
                
                # Then encode with our custom encoder
                input_data = json.loads(
                    json.dumps({"data": input_data}, cls=EnhancedJSONEncoder)
                )
            except json.JSONDecodeError as e:
                raise ValueError(f"Invalid JSON input data: {e}")

        workflow_handle = await self.temporal_client.start_workflow(
            "ScriptWorkflow",
            args=[f"{os.getcwd()}/app/scripts/{name}", input_data],
            id=name,
            task_queue="python-script-queue",
            retry_policy=retry_policy,
            run_timeout=run_timeout
        )

        return workflow_handle.result_run_id

    def load_consolidated_configs(self):
        try:
            file_path = os.path.join(os.getcwd(), ".moose", "workflow_configs.json")
            with open(file_path, 'r') as file:
                data = json.load(file)
                config_map = {config['name']: config for config in data}
                return config_map
        except Exception as e:
            raise ValueError(f"Error loading file {file_path}: {e}")

    def parse_timeout_to_timedelta(self, timeout_str: str) -> timedelta:
        if timeout_str.endswith('h'):
            return timedelta(hours=int(timeout_str[:-1]))
        elif timeout_str.endswith('m'):
            return timedelta(minutes=int(timeout_str[:-1]))
        elif timeout_str.endswith('s'):
            return timedelta(seconds=int(timeout_str[:-1]))
        else:
            raise ValueError(f"Unsupported timeout format: {timeout_str}")

class MooseClient:
    def __init__(self, ch_client: ClickhouseClient, temporal_client: Optional[TemporalClient] = None):
        self.query = QueryClient(ch_client)
        if temporal_client:
            self.workflow = WorkflowClient(temporal_client)
        else:
            self.workflow = None

def verify_jwt(token: str) -> Optional[Dict[str, Any]]:
    try:
        payload = jwt.decode(token, jwt_secret, algorithms=["RS256"], audience=jwt_audience, issuer=jwt_issuer)
        return payload
    except Exception as e:
        print("JWT verification failed:", str(e))
        return None

def has_jwt_config() -> bool:
    return jwt_secret and jwt_issuer and jwt_audience

def handler_with_client(moose_client):
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
                            query_params = map_params_to_class(query_params, fields, cls)
                        response = module.run(moose_client, query_params, jwt_payload)
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
                        query_params = map_params_to_class(query_params, fields, cls)
                    response = module.run(moose_client, query_params)
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

async def get_temporal_client():
    return await TemporalClient.connect("localhost:7233")

def main():
    print(f"Connecting to Clickhouse at {interface}://{host}:{port}")
    ch_client = get_client(interface=interface, host=host,
                           port=port, database=db, username=user, password=password)

    temporal_client = None
    try:
        # TODO: try to connect since it's still behind a feature flag
        print("Connecting to Temporal")
        # Need to await on temporal client calls but this main function is sync
        temporal_client = asyncio.run(get_temporal_client())
    except Exception as e:
        print(f"Failed to connect to Temporal. Is the feature flag enabled? {e}")
    
    moose_client = MooseClient(ch_client, temporal_client)

    server_address = ('', 4001)
    handler = handler_with_client(moose_client)

    httpd = HTTPServer(server_address, handler)
    print(f"Starting server on http://localhost:4001")
    httpd.serve_forever()


main()
