from dataclasses import dataclass
from datetime import timedelta
from moose_lib.dmv2 import get_workflow, Workflow, Task
from temporalio import workflow
from temporalio.common import RetryPolicy
from typing import Any, Dict, List, Optional
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
                ScriptExecutionInput(dmv2_workflow_name=dmv2wf.name, task_name=task.name, input_data=input_data),
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
    async def run(self, workflow_name: str, input_data: Optional[Dict] = None) -> List[WorkflowStepResult]:
        """Execute a DMv2 workflow by name.

        Args:
            workflow_name: Name of the DMv2 workflow to execute
            input_data: Optional input data for the workflow

        Returns:
            List of workflow step results

        Raises:
            ValueError: If workflow is not found or input data is invalid
        """
        log.info(f"Starting DMv2 workflow: {workflow_name} with input: {input_data}")

        # Process input data
        current_data = {}
        if input_data:
            try:
                current_data = input_data.get("data", input_data)
                current_data = json.loads(
                    json.dumps(current_data),
                    object_hook=moose_json_decode
                )
                log.info(f"Processed input data: {current_data}")
            except Exception as e:
                log.error(f"Failed to decode input data: {e}")
                raise ValueError(f"Invalid input data: {e}")
        
        # Get DMv2 workflow
        dmv2wf = get_workflow(workflow_name)
        if not dmv2wf:
            raise ValueError(f"DMv2 workflow '{workflow_name}' not found")

        log.info(f"Executing DMv2 workflow: {dmv2wf.name}")
        return await self._execute_dmv2_activity_with_state(dmv2wf, dmv2wf.config.starting_task, current_data)
