from temporalio import activity
from dataclasses import dataclass
from moose_lib.dmv2 import get_workflow
from typing import Optional, Callable
import asyncio
import json
import traceback
import concurrent.futures

from .logging import log
from .types import WorkflowStepResult
from .serialization import moose_json_decode

@dataclass
class ScriptExecutionInput:
    dmv2_workflow_name: str
    task_name: str
    input_data: Optional[dict] = None


async def _create_heartbeat_task(task_name: str) -> asyncio.Task:
    """Create a heartbeat task for cancellation detection."""
    async def heartbeat_loop():
        while True:
            try:
                activity.heartbeat(f"Task {task_name} in progress")
                await asyncio.sleep(5)
            except asyncio.CancelledError:
                log.info(f"Heartbeat loop cancelled for task {task_name}")
                break

    return asyncio.create_task(heartbeat_loop())


def _process_input_data(execution_input: ScriptExecutionInput) -> dict:
    """Process and validate input data."""
    return execution_input.input_data.get('data', execution_input.input_data) if execution_input.input_data else {}


def _get_workflow_and_task(execution_input: ScriptExecutionInput):
    """Get workflow and task objects from execution input."""
    log.info(f"Getting workflow {execution_input.dmv2_workflow_name}")
    workflow = get_workflow(execution_input.dmv2_workflow_name)
    if not workflow:
        raise ValueError(f"Workflow {execution_input.dmv2_workflow_name} not found")

    log.info(f"Getting task {execution_input.task_name} from workflow {execution_input.dmv2_workflow_name}")
    task = workflow.get_task(execution_input.task_name)
    if not task:
        raise ValueError(f"Task {execution_input.task_name} not found in workflow {execution_input.dmv2_workflow_name}")

    return workflow, task


def _validate_input_data(input_data: dict, task) -> any:
    """Validate input data against task's input type."""
    if input_data:
        try:
            validated_data = task.model_type.model_validate(input_data)
            log.info(f"Converted input data to {task.model_type.__name__}: {validated_data}")
            return validated_data
        except Exception as e:
            log.error(f"Failed to validate input data against {task.model_type.__name__}: {e}")
            raise ValueError(f"Input data does not match task's input type {task.model_type.__name__}: {e}")
    return input_data


async def _execute_task_function(task, input_data, executor) -> any:
    """Execute the task function (sync or async)."""
    task_func = task.config.run

    if asyncio.iscoroutinefunction(task_func):
        if input_data:
            return await task_func(input=input_data)
        else:
            return await task_func()
    else:
        # Run blocking sync function in thread executor
        loop = asyncio.get_running_loop()
        if input_data:
            future = loop.run_in_executor(executor, lambda: task_func(input=input_data))
        else:
            future = loop.run_in_executor(executor, task_func)

        return await asyncio.wait_for(future, timeout=None)


async def _handle_task_cancellation(task, task_name: str):
    """Handle task cancellation and call onCancel handler if it exists."""
    log.info(f"Task {task_name} cancelled, calling onCancel handler if it exists")

    if task.config.on_cancel:
        try:
            if asyncio.iscoroutinefunction(task.config.on_cancel):
                await task.config.on_cancel()
            else:
                task.config.on_cancel()
            log.info(f"onCancel handler completed for task {task_name}")
        except Exception as cancel_error:
            log.error(f"Error in onCancel handler for task {task_name}: {cancel_error}")
            # Don't re-raise onCancel errors, just log them


async def _cleanup_resources(heartbeat_task: asyncio.Task, executor):
    """Clean up heartbeat task and executor."""
    heartbeat_task.cancel()
    try:
        await heartbeat_task
    except asyncio.CancelledError:
        pass  # Expected when we cancel the heartbeat task
    executor.shutdown(wait=False)

def create_activity_for_script(script_name: str) -> Callable:
    """Return a new Activity function whose ActivityType = script_name."""
    
    @activity.defn(name=script_name)
    async def dynamic_activity(execution_input: ScriptExecutionInput) -> WorkflowStepResult:
        """Execute a DMv2 task with cancellation support."""
        return await _execute_dmv2_task(execution_input, script_name)

    return dynamic_activity


async def _execute_dmv2_task(execution_input: ScriptExecutionInput, script_name: str) -> WorkflowStepResult:
    """Main task execution logic separated for better testability and readability."""
    executor = concurrent.futures.ThreadPoolExecutor(max_workers=1)
    heartbeat_task = await _create_heartbeat_task(execution_input.task_name)

    try:
        log.info(f"Executing DMv2 task {script_name} with input {execution_input}")

        # Process and validate input
        input_data = _process_input_data(execution_input)
        log.info(f"Processed input_data for task: {input_data}")

        # Get workflow and task objects
        workflow, task = _get_workflow_and_task(execution_input)
        log.info(f"Found task {execution_input.task_name} in workflow {execution_input.dmv2_workflow_name}")

        # Validate input data against task's input type
        validated_input = _validate_input_data(input_data, task)

        # Send initial heartbeat
        activity.heartbeat(f"Starting task: {execution_input.task_name}")

        try:
            # Execute the task function
            result = await _execute_task_function(task, validated_input, executor)

            # Return structured result
            return WorkflowStepResult(
                task=execution_input.task_name,
                data=result
            )
        except asyncio.CancelledError:
            # Handle cancellation and call onCancel handler
            await _handle_task_cancellation(task, execution_input.task_name)
            raise  # Re-raise to signal cancellation to Temporal

    except Exception as e:
        if not isinstance(e, asyncio.CancelledError):
            # Handle non-cancellation errors
            error_data = {
                "error": "Task execution failed",
                "details": str(e),
                "traceback": traceback.format_exc(),
            }
            log.error(json.dumps(error_data))
            from temporalio.exceptions import ApplicationError
            raise ApplicationError(json.dumps(error_data))
        else:
            raise  # Re-raise CancelledError
    finally:
        # Clean up resources
        await _cleanup_resources(heartbeat_task, executor)