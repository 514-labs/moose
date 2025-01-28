from temporalio.client import Client
from temporalio.worker import Worker
import os
from typing import List, Optional
from .workflow import ScriptWorkflow
from .activity import create_activity_for_script
from .logging import log

# Maintain a global set of activity names we’ve already created
_ALREADY_REGISTERED = set()

EXCLUDE_DIRS = [".moose"]

def collect_activities(workflow_dir: str) -> List[str]:
    """
    Recursively collect all Python files from the workflow directory and its subdirectories,
    """
    script_paths = []
    for root, _, files in os.walk(workflow_dir):
        # Skip any folders named 'python_wrapper'
        for exclude_dir in EXCLUDE_DIRS:
            if exclude_dir in root:
                continue

        # We need it sorted to ensure that the activities are registered in the correct order
        # This is important for the workflow to execute the activities in the correct order
        for file in sorted(files):
            if file.endswith(".py"):
                script_paths.append(os.path.join(root, file))

    return script_paths

async def register_workflows(script_dir: str) -> Optional[Worker]:
    """
    Register all workflows and activities without executing them.
    Activity names should match the format: "parent_dir/script_name"
    """
    # Collect all Python scripts, deduplicating if needed
    all_script_paths = []

    # As all workflows are defined as root directories under the scripts directory, all the direct 
    # children of the scripts directory are workflows.
    for workflow_dir in os.listdir(script_dir):
        
        workflow_dir_full_path = os.path.join(script_dir, workflow_dir)

        if os.path.isdir(workflow_dir_full_path):
            # Within a workflow, all the scripts are activities.
            all_script_paths.extend(collect_activities(workflow_dir_full_path))

    if len(all_script_paths) == 0:
        log.info(f"No Python scripts found in {script_dir}")
        return None

    log.info(f"Found {len(all_script_paths)} Python scripts in {script_dir}")

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
            log.info(f"Registered activity {activity_name}")

    if len(dynamic_activities) == 0:
        log.info(f"No activities found in {script_dir}")
        return None
    else:
        log.info(f"Found {len(dynamic_activities)} activity(ies) in {script_dir}")

    # TODO: This should be configurable
    client = await Client.connect("localhost:7233")
    worker = Worker(
        client,
        # All the workflows are registered under the same task queue for now
        task_queue="python-script-queue",
        # This is interesting, the workflow will be different based on the file system. 
        # The docs says the Worklow should be deterministic. I was expecting multiple workflows
        # to be registered under the same task queue. One for each arborescence under the scripts directory.
        # This seems to work, but I'm not sure if it's the most reliable approach.
        workflows=[ScriptWorkflow],
        activities=dynamic_activities,
    )
    return worker

async def start_worker(script_dir: str) -> Worker:
    """
    Start a Temporal worker that handles Python script execution workflows.

    Args:
        script_dir (str): Root directory containing Python scripts to register as activities.
                          Scripts will be registered with activity names in the format "parent_dir/script_name".

    Returns:
        Worker: The started Temporal worker instance.
    """
    worker = await register_workflows(script_dir)

    if worker is None:
        return None
    
    log.info("Python worker started")
    await worker.run()

    return worker