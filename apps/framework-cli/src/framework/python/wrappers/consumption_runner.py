import argparse
import asyncio
import dataclasses
import json
import os
import subprocess
import sys
import traceback
import signal
import threading
from datetime import datetime, timezone, date, timedelta

from http import HTTPStatus
from http.server import HTTPServer, BaseHTTPRequestHandler

from importlib import import_module
from typing import Optional, Dict, Any
from urllib.parse import urlparse, parse_qs
from moose_lib import MooseClient
from moose_lib.query_param import map_params_to_class, convert_consumption_api_param, convert_pydantic_definition
from moose_lib.internal import load_models
from moose_lib.dmv2 import get_consumption_api, get_workflow
from pydantic import BaseModel, ValidationError

import jwt
from clickhouse_connect import get_client

from moose_lib.commons import EnhancedJSONEncoder

from consumption_wrapper.utils import create_temporal_connection

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
parser.add_argument('temporal_url', type=str, help='Temporal URL')
parser.add_argument('client_cert', type=str, help='Client certificate')
parser.add_argument('client_key', type=str, help='Client key')
parser.add_argument('api_key', type=str, help='API key')
parser.add_argument('is_dmv2', type=str, help='Is DMv2')
parser.add_argument('proxy_port', type=int, help='Proxy port')

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

temporal_url = args.temporal_url
client_cert = args.client_cert
client_key = args.client_key
api_key = args.api_key
is_dmv2 = args.is_dmv2.lower() == 'true'

sys.path.append(consumption_dir_path)


def verify_jwt(token: str) -> Optional[Dict[str, Any]]:
    try:
        payload = jwt.decode(token, jwt_secret, algorithms=["RS256"], audience=jwt_audience, issuer=jwt_issuer)
        return payload
    except Exception as e:
        print("JWT verification failed:", str(e))
        return None

def has_jwt_config() -> bool:
    return jwt_secret and jwt_issuer and jwt_audience

def is_dmv2_enabled() -> bool:
    return is_dmv2

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
            path_parts = parsed_path.path.lstrip('/').split('/')
            
            # Parse version from URL path (e.g., "bar/1" -> version="1", endpoint="bar")
            module_name = path_parts[0]
            version_from_path = None
            if len(path_parts) >= 2:
                version_from_path = path_parts[1]
            
            # Extract version information early and maintain it consistently
            # Check for version in the custom header sent by the Rust webserver
            version_from_header = self.headers.get('x-moose-api-version')
            
            # Use version from path first, then fallback to header
            request_version = version_from_path or version_from_header
            
            # Generate versioned module name consistently
            versioned_module_name = None
            if request_version:
                versioned_module_name = f"v{request_version}/{module_name}"
            
            # Parse JWT token
            jwt_payload = None
            auth_header = self.headers.get('Authorization')
            if auth_header:
                try:
                    bearer_token = auth_header.split(' ')[1] if auth_header.startswith('Bearer ') else None
                    if bearer_token:
                        jwt_payload = jwt.decode(bearer_token, options={"verify_signature": False})
                except:
                    jwt_payload = None

            # Parse query parameters
            query_params = {}
            try:
                query_params = parse_qs(parsed_path.query, keep_blank_values=True)
                # Keep all values as lists - map_params_to_class expects this format
            except Exception as e:
                print(f"Error parsing query params: {e}")

            try:
                # Check if we have a registered consumption API
                if is_dmv2_enabled():
                    # Improved deterministic version resolution logic
                    api_to_use = None
                    if request_version:
                        # When a specific version is requested, only look for that exact version
                        api_to_use = get_consumption_api(module_name, request_version)
                        
                        if not api_to_use:
                            # Version was explicitly requested but not found - this is an error condition
                            print(f"Warning: Explicitly requested version {request_version} for endpoint {module_name} not found")
                    else:
                        # When no version is specified, try unversioned first
                        api_to_use = get_consumption_api(module_name)
                        
                        if not api_to_use:
                            # Try to find any versioned implementation as fallback
                            # Use a more deterministic approach: find all versions and pick the highest one
                            from ._registry import _egress_apis
                            import re
                            
                            available_versions = []
                            pattern = re.compile(rf"^v(\d+(?:\.\d+)*)\/{re.escape(module_name)}$")
                            
                            for api_key in _egress_apis.keys():
                                match = pattern.match(api_key)
                                if match:
                                    available_versions.append(match.group(1))
                            
                            if available_versions:
                                # Sort versions and pick the highest one for deterministic behavior
                                def version_key(version_str):
                                    return tuple(int(x) for x in version_str.split('.'))
                                
                                available_versions.sort(key=version_key, reverse=True)
                                highest_version = available_versions[0]
                                api_to_use = get_consumption_api(module_name, highest_version)
                                
                                if api_to_use:
                                    print(f"Info: No unversioned endpoint found for {module_name}, using version {highest_version} as fallback")
                    
                    if api_to_use is not None:
                        query_fields = convert_pydantic_definition(api_to_use.model_type)
                        try:
                            params = map_params_to_class(query_params, query_fields, api_to_use.model_type)
                        except (ValidationError, ValueError) as e:
                            traceback.print_exc()
                            self.send_response(400)
                            self.end_headers()
                            self.wfile.write(str(e).encode())
                            return
                        args = [moose_client, params]
                        if jwt_payload is not None:
                            args.append(jwt_payload)
                        response = api_to_use.query_function(*args)
                        # Convert Pydantic model to dict before JSON serialization
                        if isinstance(response, BaseModel):
                            response = response.model_dump_json()
                    else:
                        self.send_response(404)
                        self.end_headers()
                        self.wfile.write(bytes(json.dumps({"error": "API not found"}), 'utf-8'))
                        return
                else:
                    # Try to import the module, maintaining version consistency
                    try:
                        if request_version:
                            # For versioned paths, try importing with version-specific module
                            versioned_import_path = f"{module_name}_v{request_version.replace('.', '_')}"
                            try:
                                module = import_module(versioned_import_path)
                            except ModuleNotFoundError:
                                # Try with just the major version (v1, v2, etc.)
                                major_version = request_version.split('.')[0]
                                versioned_import_path = f"{module_name}_v{major_version}"
                                try:
                                    module = import_module(versioned_import_path)
                                except ModuleNotFoundError:
                                    # Fall back to unversioned module if versioned not found
                                    module = import_module(module_name)
                        else:
                            # No version specified, try regular import
                            module = import_module(module_name)
                    except ModuleNotFoundError:
                        self.send_response(404)
                        self.end_headers()
                        error_msg = f"API module not found: {module_name}"
                        if request_version:
                            error_msg += f" with version {request_version}"
                        self.wfile.write(bytes(json.dumps({"error": error_msg}), 'utf-8'))
                        return
                        
                    fields_and_class = convert_consumption_api_param(module)

                    if fields_and_class is not None:
                        (cls, fields) = fields_and_class
                        query_params = map_params_to_class(query_params, fields, cls)
                        args = [moose_client, query_params]
                        if jwt_payload is not None:
                            args.append(jwt_payload)
                        response = module.run(*args)
                    else:
                        self.send_response(404)
                        self.end_headers()
                        self.wfile.write(bytes(json.dumps({"error": "No consumption API found"}), 'utf-8'))
                        return

                self.send_response(200)
                self.send_header('Content-type', 'application/json')
                self.send_header('Access-Control-Allow-Origin', '*')
                self.send_header('Access-Control-Allow-Methods', 'GET, POST')
                self.send_header('Access-Control-Allow-Headers', 'Authorization, Content-Type, baggage, sentry-trace, traceparent, tracestate')
                self.end_headers()

                if isinstance(response, str):
                    self.wfile.write(response.encode())
                else:
                    self.wfile.write(bytes(json.dumps(response), 'utf-8'))

            except Exception as e:
                traceback.print_exc()
                self.send_response(500)
                self.end_headers()
                error_response = json.dumps({"error": str(e)})
                self.wfile.write(error_response.encode())

        def do_OPTIONS(self):
            self.send_response(200)
            self.send_header('Access-Control-Allow-Origin', '*')
            self.send_header('Access-Control-Allow-Methods', 'GET, POST')
            self.send_header('Access-Control-Allow-Headers', 'Authorization, Content-Type, baggage, sentry-trace, traceparent, tracestate')
            self.end_headers()

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

    temporal_client = None
    try:
        print("Connecting to Temporal")
        temporal_client = asyncio.run(create_temporal_connection(temporal_url, client_cert, client_key, api_key))
    except Exception as e:
        print(f"Failed to connect to Temporal. Is the feature flag enabled? {e}")

    if is_dmv2:
        print("Loading DMv2 models")
        load_models()

    moose_client = MooseClient(ch_client, temporal_client)
    server_port = args.proxy_port
    server_address = ('localhost', server_port)
    handler = handler_with_client(moose_client)
    httpd = HTTPServer(server_address, handler)
    
    # Store references for cleanup
    httpd.moose_client = moose_client
    
    def shutdown_server():
        httpd.shutdown()
        print("\nShutting down server...")
        httpd.server_close()
        # Cleanup clients
        asyncio.run(moose_client.cleanup())
        print("Server shutdown complete")
    
    def signal_handler(signum, frame):
        print(f"\nReceived signal {signum}. Starting graceful shutdown...")
        # Start shutdown in a separate thread to avoid deadlock
        threading.Thread(target=shutdown_server).start()
    
    # Register signal handlers
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    signal.signal(signal.SIGQUIT, signal_handler)
    signal.signal(signal.SIGHUP, signal_handler)
    
    print(f"Starting server on http://localhost:{server_port}")
    
    try:
        httpd.serve_forever()
    except Exception as e:
        print(f"Server error: {e}")


main()
