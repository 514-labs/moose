from temporalio import workflow, activity
import os
import sys
import json
import traceback
import importlib.util
from datetime import timedelta
from typing import Any, Optional, List, Callable
from dataclasses import dataclass
import asyncio

#
# 1) We define a factory function that creates an Activity with the desired name.
#
def create_activity_for_script(script_name: str):
    """Return a new Activity function whose ActivityType = script_name."""
    @activity.defn(name=script_name)
    async def dynamic_activity(execution_input: "ScriptExecutionInput") -> Any:
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

@dataclass
class ScriptExecutionInput:
    script_path: str
    input_data: Optional[dict] = None

@workflow.defn(sandboxed=False)  # We use sandboxed=False to simplify pass-through usage
class ScriptWorkflow:
    @workflow.run
    async def run(self, script_path: str, input_data: Optional[dict] = None) -> List[Any]:
        """Workflow that runs either one file or multiple files (including parallel dirs)."""
        results = []
        
        # Single file? We just call the matching Activity
        if os.path.isfile(script_path) and script_path.endswith(".py"):
            activity_name = os.path.splitext(os.path.basename(script_path))[0]
            single_result = await workflow.execute_activity(
                activity_name,  # use the generated ActivityType
                ScriptExecutionInput(script_path, input_data),
                start_to_close_timeout=timedelta(minutes=10),
            )
            return [single_result]
        
        # If it's a directory, handle both sequential and parallel
        parallel_tasks: List[asyncio.Future] = []
        for item in sorted(os.listdir(script_path)):
            full_path = os.path.join(script_path, item)
            
            # If top-level .py => run sequentially
            if os.path.isfile(full_path) and item.endswith(".py"):
                act_name = os.path.splitext(item)[0]
                single_result = await workflow.execute_activity(
                    act_name,
                    ScriptExecutionInput(full_path, input_data),
                    start_to_close_timeout=timedelta(minutes=10),
                )
                results.append(single_result)
            # If a parallel dir => gather tasks
            elif os.path.isdir(full_path) and "parallel" in item:
                files = [f for f in sorted(os.listdir(full_path)) if f.endswith(".py")]
                for script_file in files:
                    script_full = os.path.join(full_path, script_file)
                    act_name = os.path.splitext(script_file)[0]
                    parallel_tasks.append(
                        workflow.execute_activity(
                            act_name,
                            ScriptExecutionInput(script_full, input_data),
                            start_to_close_timeout=timedelta(minutes=10),
                        )
                    )
        
        # Await parallel tasks
        if parallel_tasks:
            parallel_results = await asyncio.gather(*parallel_tasks)
            results.extend(parallel_results)
        
        return results


# In run_workflow, we generate Activity definitions for each .py script in the top-level folder (recursively),
# then register them. We name each Activity with the base name of the file.
#
import glob
from temporalio.client import Client
from temporalio.worker import Worker
from temporalio.exceptions import WorkflowAlreadyStartedError

async def run_workflow(script_path: str, input_data: Optional[dict] = None) -> None:
    # Use the top-level directory name (or parent) as the workflow ID
    if os.path.isdir(script_path):
        workflow_id = os.path.basename(script_path)
    else:
        workflow_id = os.path.basename(os.path.dirname(script_path))
    
    # Gather .py files from this folder (and children) so we can pre-register them
    # (Temporal requires Activities to be registered before we can call them by name.)
    script_paths = []
    if os.path.isfile(script_path) and script_path.endswith(".py"):
        # Just the single file
        script_paths = [script_path]
    else:
        # Collect all .py files in subdirs
        for root, dirs, files in os.walk(script_path):
            for f in files:
                if f.endswith(".py"):
                    script_paths.append(os.path.join(root, f))
    
    # Build a dynamic activity list
    dynamic_activities = []
    for p in script_paths:
        base_name = os.path.splitext(os.path.basename(p))[0]
        act = create_activity_for_script(base_name)  # pass the base name as the activity name
        dynamic_activities.append(act)

    client = await Client.connect("localhost:7233")
    try:
        async with Worker(
            client,
            task_queue="python-script-queue",
            workflows=[ScriptWorkflow],
            activities=dynamic_activities,  # Register all dynamic activities 
        ):
            result = await client.execute_workflow(
                ScriptWorkflow.run,
                args=[script_path, input_data],
                id=workflow_id,
                task_queue="python-script-queue",
            )
            if result is not None:
                print(json.dumps(result))
    except WorkflowAlreadyStartedError:
        handle = client.get_workflow_handle(workflow_id)
        print(
            json.dumps({
                "status": "workflow_running",
                "message": f"Workflow {workflow_id} is already running",
                "workflow_id": workflow_id
            }),
            file=sys.stderr
        )
        sys.exit(1)

def main():
    script_path = sys.argv[1]
    input_data = (
        json.loads(os.environ["MOOSE_INPUT"]) if "MOOSE_INPUT" in os.environ else None
    )
    import asyncio
    asyncio.run(run_workflow(script_path, input_data))

if __name__ == "__main__":
    main()