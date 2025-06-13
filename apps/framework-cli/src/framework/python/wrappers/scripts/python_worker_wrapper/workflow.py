from dataclasses import dataclass
from datetime import timedelta
from moose_lib.dmv2 import get_workflow, Workflow, Task
from temporalio import workflow
from temporalio.common import RetryPolicy
from typing import Any, Dict, List, Optional
import asyncio
import os
import importlib.util
import sys
from .activity import ScriptExecutionInput
from .logging import log
from .types import WorkflowStepResult
from .serialization import moose_json_decode
import json
import humanfriendly

# TODO: make this configurable
START_TO_CLOSE_TIMEOUT_MINUTES = 60 


# TODO: make this configurable
START_TO_CLOSE_TIMEOUT_MINUTES = 60 


@dataclass
class WorkflowState:
    """Represents the current state of a script workflow execution.
    
    Attributes:
        completed_steps: List of activity names that have completed successfully
        current_step: Name of the activity currently being executed, if any
        failed_step: Name of the activity that failed, if any
        script_path: Path to the script file being executed
        input_data: Optional input data passed to the script
    """
    completed_steps: List[str]
    current_step: Optional[str]
    failed_step: Optional[str]
    script_path: Optional[str]
    input_data: Optional[Dict]

@workflow.defn(sandboxed=False)
class ScriptWorkflow:
    """A Temporal workflow that executes Python scripts as activities.
    
    This workflow can:
    - Execute a single Python script
    - Execute multiple scripts in sequence from a directory
    - Execute scripts in parallel from specially named directories
    - Execute child workflows based on directory structure
    """
    def __init__(self):
        self._state = WorkflowState(
            completed_steps=[],
            current_step=None,
            failed_step=None,
            script_path=Optional[str],
            input_data=None,
        )

    @workflow.signal(name="resume")
    async def resume_execution(self) -> Dict:
        """Signal to resume workflow after a step has been fixed.
        
        Clears the failed step state and continues execution with the same script path
        and input data.
        
        Returns:
            Dict: Result of the continued workflow execution
        """
        self._state.failed_step = None
        script_path = self._state.script_path
        input_data = self._state.input_data

        return await workflow.continue_as_new(args=[script_path, input_data])

    @workflow.query
    def get_workflow_state(self) -> WorkflowState:
        """Query current workflow state.
        
        Returns:
            WorkflowState: Current state of the workflow execution
        """
        return self._state

    def _get_activity_retry(self, path: str) -> int:
        """Load a Python script and get the retry argument."""
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
        
        retries = 1
        for attr_name in dir(module):
            attr = getattr(module, attr_name)
            if hasattr(attr, "_retries"):
                retries = getattr(attr, "_retries")
                log.info(f"Using retries in {attr_name}: {retries}")
                break
        
        return retries

    async def _execute_activity_with_state(
        self, 
        activity_name: str, 
        script_path: str, 
        input_data: Optional[Dict] = None
    ) -> Any:
        """Execute a single activity while maintaining workflow state.
        
        Args:
            activity_name: Name of the activity to execute
            script_path: Path to the Python script to run
            input_data: Optional data to pass to the script
            
        Returns:
            Any: Result returned by the activity
            
        Raises:
            Exception: If the activity execution fails
        """
        self._state.current_step = activity_name
        
        try:
            retries = self._get_activity_retry(script_path)
            result = await workflow.execute_activity(
                activity_name,
                ScriptExecutionInput(script_path=script_path, input_data=input_data),
                start_to_close_timeout=timedelta(minutes=START_TO_CLOSE_TIMEOUT_MINUTES),
                retry_policy=RetryPolicy(
                    maximum_attempts=retries,
                ),
            )
            self._state.completed_steps.append(activity_name)
            return result
        except Exception as e:
            self._state.failed_step = activity_name
            # Re-raise the exception to maintain existing error handling
            raise

    async def _execute_dmv2_activity_with_state(self, dmv2wf: Workflow, task: Task, input_data: Optional[Dict] = None) -> Any:
        activity_name = f"{dmv2wf.name}/{task.name}"
        self._state.current_step = activity_name
        results = []

        try:
            timeout = timedelta(seconds=humanfriendly.parse_timespan(task.config.timeout)) if task.config.timeout else timedelta(minutes=START_TO_CLOSE_TIMEOUT_MINUTES)
            retries = task.config.retries or 3
            log.info(f"<DMV2WF> Executing activity {activity_name} with timeout {timeout} and retries {retries}")
            result = await workflow.execute_activity(
                activity_name,
                ScriptExecutionInput(dmv2_workflow_name=dmv2wf.name, script_path=task.name, input_data=input_data),
                start_to_close_timeout=timeout,
                retry_policy=RetryPolicy(
                    maximum_attempts=retries,
                ),
            )
            results.append(result)
            self._state.completed_steps.append(activity_name)

            if task.config.on_complete:
                for child_task in task.config.on_complete:
                    child_result = await self._execute_dmv2_activity_with_state(dmv2wf, child_task, result)
                    results.extend(child_result)

            return results
        except Exception as e:
            self._state.failed_step = activity_name
            raise

    @workflow.run
    async def run(self, path: str, input_data: Optional[Dict] = None) -> List[WorkflowStepResult]:
        results = []

        # Add logging to see what input data we receive
        log.info(f"Initial input_data received for workflow: {path} : {input_data}")

        current_data = {}
        if input_data:
            try:
                current_data = input_data.get("data", input_data)
                current_data = json.loads(
                    json.dumps(current_data),
                    object_hook=moose_json_decode
                )
                log.info(f"Processed current_data: {current_data}")
            except Exception as e:
                log.error(f"Failed to decode input data: {e}")
                raise
        
        dmv2wf = get_workflow(path)
        if dmv2wf:
            return await self._execute_dmv2_activity_with_state(dmv2wf, dmv2wf.config.starting_task, current_data)
        else:
            if os.path.isfile(path) and path.endswith(".py"):
                # Single script execution
                activity_name = f"{os.path.basename(os.path.dirname(path))}/{os.path.splitext(os.path.basename(path))[0]}"
                result = await self._execute_activity_with_state(activity_name, path, current_data)
                return [result]

            # Sequential execution with data passing
            for item in sorted(os.listdir(path)):
                full_path = os.path.join(path, item)

                if os.path.isfile(full_path) and item.endswith(".py"):
                    activity_name = f"{os.path.basename(os.path.dirname(full_path))}/{os.path.splitext(item)[0]}"
                    result = await self._execute_activity_with_state(activity_name, full_path, current_data)
                    results.append(result)
                    current_data = result  # Pass result to next step

                elif os.path.isdir(full_path) and "parallel" in item:
                    # Handle parallel execution
                    parallel_tasks = []
                    for script_file in sorted(os.listdir(full_path)):
                        if script_file.endswith(".py"):
                            script_path = os.path.join(full_path, script_file)
                            activity_name = f"{os.path.basename(full_path)}/{os.path.splitext(script_file)[0]}"
                            task = self._execute_activity_with_state(activity_name, script_path, current_data)
                            parallel_tasks.append(task)

                    if parallel_tasks:
                        parallel_results = await asyncio.gather(*parallel_tasks)
                        results.extend(parallel_results)
                        # Merge parallel results if needed
                        current_data = {
                            "data": {k: v for result in parallel_results for k, v in result["data"].items()}
                        }

        return results
