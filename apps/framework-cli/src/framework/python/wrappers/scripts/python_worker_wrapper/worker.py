from temporalio.client import Client
from temporalio.worker import Worker
import os
from typing import List, Optional
from .workflow import ScriptWorkflow
from .activity import create_activity_for_script
from .logging import log
import asyncio
import signal
from .utils.temporal import create_temporal_connection

# Maintain a global set of activity names we've already created
_ALREADY_REGISTERED = set()

EXCLUDE_DIRS = [".moose"]

def collect_activities(workflow_dir: str) -> List[str]:
    """
    Recursively collect all Python files from the workflow directory and its subdirectories,
    """
    log.info(f"Collecting tasks from {workflow_dir}")
    script_paths = []
    for root, _, files in os.walk(workflow_dir):
        # Skip any folders named 'python_wrapper'
        for exclude_dir in EXCLUDE_DIRS:
            if exclude_dir in root:
                log.debug(f"Skipping excluded directory: {root}")
                continue

        # We need it sorted to ensure that the activities are registered in the correct order
        # This is important for the workflow to execute the activities in the correct order
        for file in sorted(files):
            if file.endswith(".py"):
                script_path = os.path.join(root, file)
                script_paths.append(script_path)
                log.debug(f"Found script: {script_path}")

    log.info(f"Found {len(script_paths)} scripts")
    return script_paths

async def register_workflows(temporal_url: str, script_dir: str, client_cert: str, client_key: str, api_key: str) -> Optional[Worker]:
    """
    Register all workflows and activities without executing them.
    Activity names should match the format: "parent_dir/script_name"
    """
    log.info(f"Registering workflows from {script_dir}")
    
    # Collect all Python scripts, deduplicating if needed
    all_script_paths = []

    try:
        # As all workflows are defined as root directories under the scripts directory, all the direct 
        # children of the scripts directory are workflows.
        for workflow_dir in os.listdir(script_dir):
            workflow_dir_full_path = os.path.join(script_dir, workflow_dir)
            log.debug(f"Checking workflow directory: {workflow_dir_full_path}")

            if os.path.isdir(workflow_dir_full_path):
                # Within a workflow, all the scripts are activities.
                all_script_paths.extend(collect_activities(workflow_dir_full_path))

        if len(all_script_paths) == 0:
            log.warning(f"No Python scripts found in {script_dir}")
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
                log.info(f"Registered task {activity_name}")

        if len(dynamic_activities) == 0:
            log.warning(f"No tasks found in {script_dir}")
            return None
        else:
            log.info(f"Found {len(dynamic_activities)} task(s) in {script_dir}")

        log.info("Connecting to Temporal server...")
        client = await create_temporal_connection(temporal_url, client_cert, client_key, api_key)
        
        log.info("Creating worker...")
        worker = Worker(
            client,
            task_queue="python-script-queue",
            workflows=[ScriptWorkflow],
            activities=dynamic_activities,
        )
        log.info("Worker created successfully")
        return worker
        
    except Exception as e:
        log.error(f"Error registering workflows: {str(e)}")
        raise

async def start_worker(temporal_url: str, script_dir: str, client_cert: str, client_key: str, api_key: str) -> Worker:
    """
    Start a Temporal worker that handles Python script execution workflows.

    Args:
        script_dir (str): Root directory containing Python scripts to register as activities.
                         Scripts will be registered with activity names in the format "parent_dir/script_name".
    
    Returns:
        Worker: The started Temporal worker instance.
        
    Raises:
        ValueError: If no scripts are found to register
    """
    log.info(f"Starting worker for script directory: {script_dir}")

    loop = asyncio.get_running_loop()

    worker = await register_workflows(temporal_url, script_dir, client_cert, client_key, api_key)

    if worker is None:
        msg = f"No scripts found to register in {script_dir}"
        log.error(msg)
        raise ValueError(msg)

    shutdown_task = None

    # Define shutdown coroutine
    async def shutdown():
        log.info("Initiating graceful shutdown...")
        try:
            await asyncio.wait_for(worker.shutdown(), timeout=5.0)
        except asyncio.TimeoutError:
            log.warning("Worker shutdown timed out after 5 seconds")
        except Exception as e:
            log.error(f"Error during worker shutdown: {e}")

    # Define your signal handler
    def handle_signal(signame):
        nonlocal shutdown_task
        if shutdown_task is None:  # Prevent multiple shutdown tasks
            log.info(f"Received signal {signame}, initiating shutdown...")
            shutdown_task = asyncio.create_task(shutdown())

    # Add signal handlers
    for signame in ('SIGTERM', 'SIGQUIT', 'SIGHUP'):
        loop.add_signal_handler(getattr(signal, signame), lambda s=signame: handle_signal(s))
    
    log.info("Starting Python worker...")
    try:
        worker_task = asyncio.create_task(worker.run())
        
        # Loop and check for shutdown
        while True:
            if shutdown_task is not None:
                # Wait for shutdown to complete
                try:
                    await asyncio.wait_for(shutdown_task, timeout=10.0)
                except asyncio.TimeoutError:
                    log.error("Shutdown timed out")
                break
            await asyncio.sleep(0.1)  # Small sleep to prevent busy loop

    except Exception as e:
        log.error(f"Worker failed to start: {e}")
        raise
    finally:
        # Cancel any remaining tasks
        if shutdown_task is not None and not shutdown_task.done():
            shutdown_task.cancel()
        if not worker_task.done():
            worker_task.cancel()
            try:
                await asyncio.wait_for(worker_task, timeout=5.0)
            except asyncio.TimeoutError:
                log.warning("Worker task did not complete after cancellation")
    
    return worker