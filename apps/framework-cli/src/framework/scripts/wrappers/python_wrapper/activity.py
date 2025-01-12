from temporalio import activity
from dataclasses import dataclass
from typing import Optional, Any
import os
import sys
import json
import traceback
import importlib.util

@dataclass
class ScriptExecutionInput:
    script_path: str
    input_data: Optional[dict] = None

def create_activity_for_script(script_name: str):
    """Return a new Activity function whose ActivityType = script_name."""
    @activity.defn(name=script_name)
    async def dynamic_activity(execution_input: ScriptExecutionInput) -> Any:
        """Load and execute a single Python script."""
        try:
            path = execution_input.script_path
            if not os.path.isfile(path):
                raise ImportError(f"Expected a Python file, got directory or invalid path: {path}")
            if not path.endswith(".py"):
                raise ImportError(f"Not a Python file: {path}")
                
            spec = importlib.util.spec_from_file_location("script", path)
            if not spec or not spec.loader:
                raise ImportError(f"Could not load script: {path}")
            
            module = importlib.util.module_from_spec(spec)
            sys.modules["script"] = module
            spec.loader.exec_module(module)
            
            # Find moose_lib task
            task_func = None
            for attr_name in dir(module):
                attr = getattr(module, attr_name)
                if hasattr(attr, "_is_moose_task"):
                    task_func = attr
                    break
                
            if not task_func:
                raise ValueError("No @task function found in script.")
            
            # Execute
            if execution_input.input_data:
                return task_func(**execution_input.input_data)
            else:
                return task_func()
            
        except Exception as e:
            # Standard error output
            error_data = {
                "error": "Script execution failed",
                "details": str(e),
                "traceback": traceback.format_exc(),
            }
            print(json.dumps(error_data), file=sys.stderr)
            # Raise an ApplicationError for structured error
            from temporalio.exceptions import ApplicationError
            raise ApplicationError(json.dumps(error_data))

    return dynamic_activity