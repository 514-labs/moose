import json
import sys
from importlib import import_module
from dataclasses import asdict

from moose_lib.query_param import QueryField, convert_consumption_api_param

def consumption_api_params(module_name: str) -> list[QueryField]:
    module = import_module("app.apis." + module_name)
    converted = convert_consumption_api_param(module)
    if converted is None:
        return []
    return converted[1]

params = consumption_api_params(sys.argv[1])
print(json.dumps({"params": [asdict(param) for param in params]}))
