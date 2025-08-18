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

def create_activity_for_script(script_name: str) -> Callable:
    """Return a new Activity function whose ActivityType = script_name."""
    
    @activity.defn(name=script_name)
    async def dynamic_activity(execution_input: ScriptExecutionInput) -> WorkflowStepResult:
        """Execute a DMv2 task."""
        executor = concurrent.futures.ThreadPoolExecutor(max_workers=1)
        try:
            log.info(f"Executing DMv2 task {script_name} with input {execution_input}")

            # Process input data
            input_data = execution_input.input_data.get('data', execution_input.input_data) if execution_input.input_data else {}
            log.info(f"Processed input_data for task: {input_data}")

            # Get workflow and task
            log.info(f"Getting workflow {execution_input.dmv2_workflow_name}")
            workflow = get_workflow(execution_input.dmv2_workflow_name)
            if not workflow:
                raise ValueError(f"Workflow {execution_input.dmv2_workflow_name} not found")

            log.info(f"Getting task {execution_input.task_name} from workflow {execution_input.dmv2_workflow_name}")
            task = workflow.get_task(execution_input.task_name)
            if not task:
                raise ValueError(f"Task {execution_input.task_name} not found in workflow {execution_input.dmv2_workflow_name}")

            task_func = task.config.run
            log.info(f"Found task {execution_input.task_name} in workflow {execution_input.dmv2_workflow_name}")

            # Validate input data against task's input type
            if input_data:
                try:
                    input_data = task.model_type.model_validate(input_data)
                    log.info(f"Converted input data to {task.model_type.__name__}: {input_data}")
                except Exception as e:
                    log.error(f"Failed to validate input data against {task.model_type.__name__}: {e}")
                    raise ValueError(f"Input data does not match task's input type {task.model_type.__name__}: {e}")

            # Execute task function
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

            # Return structured result
            return WorkflowStepResult(
                task=execution_input.task_name,
                data=result
            )

        except Exception as e:
            # Standard error output
            error_data = {
                "error": "Task execution failed",
                "details": str(e),
                "traceback": traceback.format_exc(),
            }
            log.error(json.dumps(error_data))
            # Raise an ApplicationError for structured error
            from temporalio.exceptions import ApplicationError
            raise ApplicationError(json.dumps(error_data))
        finally:
            executor.shutdown(wait=False)

    return dynamic_activity