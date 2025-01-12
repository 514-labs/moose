from temporalio.client import Client
from temporalio.worker import Worker
from temporalio.exceptions import WorkflowAlreadyStartedError
import os
import json
import sys
from typing import Optional, List
from .workflow import ScriptWorkflow
from .activity import create_activity_for_script

# Maintain a global set of activity names we’ve already created
_ALREADY_REGISTERED = set()

def collect_activities(base_path: str) -> List[str]:
    """
    Recursively collect all Python files from the base path and its subdirectories,
    excluding the internal 'python_wrapper' directory and the wrapper.py file.
    """
    script_paths = []
    for root, _, files in os.walk(base_path):
        # Skip any folders named 'python_wrapper'
        if "python_wrapper" in root:
            continue

        for file in sorted(files):
            # Skip any top-level wrapper.py
            if file == "wrapper.py":
                continue
            if file.endswith(".py"):
                script_paths.append(os.path.join(root, file))
    return script_paths

async def register_workflows(root_dir: str) -> Worker:
    """
    Register all workflows and activities without executing them.
    Activity names should match the format: "parent_dir/script_name"
    """
    # Collect all Python scripts, deduplicating if needed
    all_script_paths = []
    for item in os.listdir(root_dir):
        full_path = os.path.join(root_dir, item)
        if os.path.isdir(full_path):
            all_script_paths.extend(collect_activities(full_path))

    # Build dynamic activities for all Python files, but only once per activity name
    dynamic_activities = []
    for script_path in all_script_paths:
        parent_dir = os.path.basename(os.path.dirname(script_path))
        base_name = os.path.splitext(os.path.basename(script_path))[0]
        activity_name = f"{parent_dir}/{base_name}"

        if activity_name not in _ALREADY_REGISTERED:
            act = create_activity_for_script(activity_name)
            dynamic_activities.append(act)
            _ALREADY_REGISTERED.add(activity_name)

    client = await Client.connect("localhost:7233")
    worker = Worker(
        client,
        task_queue="python-script-queue",
        workflows=[ScriptWorkflow],
        activities=dynamic_activities,
    )
    return worker

async def run_workflow(script_path: str, input_data: Optional[dict] = None) -> None:
    """
    Main entrypoint for running a workflow with a given script path and optional input.
    """
    if os.path.isdir(script_path):
        workflow_id = os.path.basename(script_path)
    else:
        workflow_id = os.path.basename(os.path.dirname(script_path))

    # The root directory is one level above script_path
    root_dir = os.path.dirname(os.path.dirname(script_path))
    worker = await register_workflows(root_dir)

    async with worker:
        client = worker.client
        try:
            result = await client.execute_workflow(
                ScriptWorkflow.run,
                args=[script_path, input_data],
                id=workflow_id,
                task_queue="python-script-queue",
            )
            if result is not None:
                print(json.dumps(result))
        except WorkflowAlreadyStartedError:
            print(
                json.dumps({
                    "status": "workflow_running",
                    "message": f"Workflow {workflow_id} is already running",
                    "workflow_id": workflow_id
                }),
                file=sys.stderr
            )
            sys.exit(1)