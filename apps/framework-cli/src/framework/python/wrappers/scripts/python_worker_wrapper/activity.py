from temporalio import activity
from dataclasses import dataclass
from typing import Optional, Any, Callable
import asyncio
import os
import sys
import json
import traceback
import importlib.util
import concurrent.futures

from .logging import log
from .types import WorkflowStepResult
from .serialization import MooseJSONEncoder, moose_json_decode


@dataclass
class ScriptExecutionInput:
    script_path: str
    input_data: Optional[dict] = None

def create_activity_for_script(script_name: str) -> Callable:
    """Return a new Activity function whose ActivityType = script_name."""
    
    @activity.defn(name=script_name)
    async def dynamic_activity(execution_input: ScriptExecutionInput) -> WorkflowStepResult:
        """Load and execute a single Python script."""
        executor = concurrent.futures.ThreadPoolExecutor(max_workers=1)
        try:
            log.info(f"Executing activity {script_name} with input {execution_input}")
            
            path = execution_input.script_path
            if not os.path.isfile(path):
                raise ImportError(f"Expected a Python file, got directory or invalid path: {path}")
            if not path.endswith(".py"):
                raise ImportError(f"Not a Python file: {path}")
                
            spec = importlib.util.spec_from_file_location("script", path)
            if not spec or not spec.loader:
                raise ImportError(f"Could not load script: {path}")
            
            module = importlib.util.module_from_spec(spec)
            log.info(f"Loaded module at {path} Successfully")

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
                raise ValueError("No @task() function found in script.")
            
            log.info(f"Task received input: {execution_input.input_data}")
            
            # Pass the input data directly if it exists
            input_data = execution_input.input_data.get('data', execution_input.input_data) if execution_input.input_data else {}
            log.info(f"Processed input_data for task: {input_data}")
            if asyncio.iscoroutinefunction(task_func):
                if input_data:
                    result = await task_func(input=input_data)
                else:
                    result = await task_func()
            else:
                # User could run blocking sync function (i.e. time.sleep)
                # so we run it in in a thread to not block the event loop
                # and let the worker's shutdown tasks clean up
                loop = asyncio.get_running_loop()
                if input_data:
                    future = loop.run_in_executor(executor, lambda: task_func(input=input_data))
                else:
                    future = loop.run_in_executor(executor, task_func)

                result = await asyncio.wait_for(future, timeout=None)
            
            # Validate and encode result
            if not isinstance(result, dict):
                raise ValueError("Task must return a dictionary with 'task' and 'data' keys")
            
            if "task" not in result or "data" not in result:
                raise ValueError("Task result must contain 'task' and 'data' keys")
            
            # Encode the result using our custom encoder
            encoded_result = json.loads(
                json.dumps(result, cls=MooseJSONEncoder)
            )
            
            return encoded_result
            
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
        finally:
            executor.shutdown(wait=False)

    return dynamic_activity