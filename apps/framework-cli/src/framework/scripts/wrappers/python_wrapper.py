import os
import sys
import json
import traceback
import importlib.util
from typing import Any

def load_script(path: str) -> Any:
    """Load a Python script as a module"""
    try:
        spec = importlib.util.spec_from_file_location("script", path)
        if not spec or not spec.loader:
            raise ImportError(f"Could not load script: {path}")
        
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module
    except Exception as e:
        print(json.dumps({
            "error": "Script load failed",
            "details": str(e),
            "traceback": traceback.format_exc()
        }), file=sys.stderr)
        sys.exit(1)

def main():
    # Get the script path from arguments
    script_path = sys.argv[1]
    
    # Load the input data from environment
    input_data = None
    if "MOOSE_INPUT" in os.environ:
        input_data = json.loads(os.environ["MOOSE_INPUT"])

    try:
        module = load_script(script_path)
        
        # Find the main function (decorated with @task)
        main_func = None
        for attr_name in dir(module):
            attr = getattr(module, attr_name)
            if hasattr(attr, "_is_moose_task"):
                main_func = attr
                break

        if not main_func:
            raise ValueError("No task function found in script")

        # Execute the function
        if input_data:
            result = main_func(**input_data)
        else:
            result = main_func()

        # Output the result as JSON
        if result is not None:
            print(json.dumps(result))

    except Exception as e:
        print(json.dumps({
            "error": "Script execution failed",
            "details": str(e),
            "traceback": traceback.format_exc()
        }), file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()