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
from .types import WorkflowStepResult
import functools

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
    async def run(self, path: str, input_data: Optional[Dict] = None) -> List[WorkflowStepResult]:
        results = []
        current_data = input_data
        
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

def task(retries: int = 3):
    def decorator(func):
        func._is_moose_task = True
        func._retries = retries
        
        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            result = func(*args, **kwargs)
            
            # Ensure proper return format
            if not isinstance(result, dict):
                raise ValueError("Task must return a dictionary with 'step' and 'data' keys")
            
            if "step" not in result or "data" not in result:
                raise ValueError("Task result must contain 'step' and 'data' keys")
                
            return result
            
        return wrapper
    return decorator