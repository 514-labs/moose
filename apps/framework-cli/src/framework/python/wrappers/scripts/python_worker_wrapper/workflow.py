from dataclasses import dataclass
from datetime import timedelta
from temporalio import workflow
from temporalio.common import RetryPolicy
from typing import Any, Dict, List, Optional
import asyncio
import os
import importlib.util
import sys
from .activity import ScriptExecutionInput
from .logging import log

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
                start_to_close_timeout=timedelta(minutes=10),
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

    @workflow.run
    async def run(self, path: str, input_data: Optional[Dict] = None) -> List[Any]:
        """Main workflow execution logic.
        
        Handles different execution patterns based on file system structure:
        - Single script execution
        - Sequential script execution from a directory
        - Parallel script execution from specially named directories
        - Child workflow execution based on directory structure
        
        Args:
            path: Path to script file or directory containing scripts
            input_data: Optional data to pass to all scripts
            
        Returns:
            List[Any]: List of results from all executed activities/workflows
        """
        results = []
        parallel_tasks = []

        log.info(f"Running workflow with path: {path} and input_data: {input_data}")
        
        if os.path.isfile(path) and path.endswith(".py"):
            parent_dir = os.path.basename(os.path.dirname(path))
            base_name = os.path.splitext(os.path.basename(path))[0]
            activity_name = f"{parent_dir}/{base_name}"
            self._state.script_path = path
            self._state.input_data = input_data
            result = await self._execute_activity_with_state(
                activity_name, path, input_data
            )
            return [result]
        
        for item in sorted(os.listdir(path)):
            full_path = os.path.join(path, item)
            
            if os.path.isfile(full_path) and item.endswith(".py"):
                parent_dir = os.path.basename(os.path.dirname(full_path))
                base_name = os.path.splitext(item)[0]
                activity_name = f"{parent_dir}/{base_name}"
                result = await self._execute_activity_with_state(
                    activity_name, full_path, input_data
                )
                results.append(result)
            
            elif os.path.isdir(full_path) and "parallel" in item:
                for script_file in sorted(os.listdir(full_path)):
                    if script_file.endswith(".py"):
                        script_full = os.path.join(full_path, script_file)
                        parent_dir = os.path.basename(os.path.dirname(script_full))
                        base_name = os.path.splitext(script_file)[0]
                        activity_name = f"{parent_dir}/{base_name}"
                        task = self._execute_activity_with_state(
                            activity_name, script_full, input_data
                        )
                        parallel_tasks.append(task)
            
            elif os.path.isdir(full_path) and os.path.isfile(os.path.join(full_path, "child.toml")):
                workflow_name = os.path.basename(full_path).split(".", 1)[1] if "." in os.path.basename(full_path) else os.path.basename(full_path)
                ScriptWorkflow.__name__ = workflow_name
                child_workflow_id = f"{workflow.info().workflow_id}:{workflow_name}"
                
                child_result = await workflow.execute_child_workflow(
                    ScriptWorkflow.run,
                    args=[os.path.join(os.path.dirname(path), workflow_name), input_data],
                    id=child_workflow_id
                )
                
                results.append(child_result)
        
        if parallel_tasks:
            parallel_results = await asyncio.gather(*parallel_tasks)
            results.extend(parallel_results)
        
        return results