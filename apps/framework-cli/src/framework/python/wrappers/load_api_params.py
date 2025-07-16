import json
import sys
from importlib import import_module
from dataclasses import asdict

from moose_lib.query_param import QueryField, convert_consumption_api_param

def consumption_api_params_and_version(module_name: str) -> tuple[list[QueryField], str | None]:
    module = import_module("app.apis." + module_name)
    converted = convert_consumption_api_param(module)
    if converted is None:
        return [], None
    
    # Extract version from the consumption API instance
    version = None
    for attr_name in dir(module):
        attr = getattr(module, attr_name)
        if hasattr(attr, 'config') and hasattr(attr.config, 'version'):
            version = attr.config.version
            break
    
    return converted[1], version

params, version = consumption_api_params_and_version(sys.argv[1])
print(json.dumps({"params": [asdict(param) for param in params], "version": version}))
