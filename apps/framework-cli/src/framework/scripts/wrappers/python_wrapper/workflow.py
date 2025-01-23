from datetime import timedelta
from temporalio import workflow
from typing import Optional, Dict, List, Any
import os
import asyncio
from dataclasses import dataclass
from .activity import ScriptExecutionInput
from .logger import initialize_logger

@dataclass
class WorkflowState:
    completed_steps: List[str]
    current_step: Optional[str]
    failed_step: Optional[str]
    script_path: Optional[str]
    input_data: Optional[Dict]

@workflow.defn(sandboxed=False)
class ScriptWorkflow:
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
        """Signal to resume workflow after a step has been fixed"""
        self._state.failed_step = None
        script_path = self._state.script_path
        input_data = self._state.input_data

        return await workflow.continue_as_new(args=[script_path, input_data])

    @workflow.query
    def get_workflow_state(self) -> WorkflowState:
        """Query current workflow state"""
        return self._state

    async def _execute_activity_with_state(
        self, 
        activity_name: str, 
        script_path: str, 
        input_data: Optional[Dict] = None
    ) -> Any:
        self._state.current_step = activity_name
        
        try:
            result = await workflow.execute_activity(
                activity_name,
                ScriptExecutionInput(script_path=script_path, input_data=input_data),
                start_to_close_timeout=timedelta(minutes=10),
            )
            self._state.completed_steps.append(activity_name)
            return result
        except Exception as e:
            self._state.failed_step = activity_name
            # Re-raise the exception to maintain existing error handling
            raise

    @workflow.run
    async def run(self, script_path: str, input_data: Optional[Dict] = None) -> List[Any]:
        workflow_id = workflow.info().workflow_id
        run_id = workflow.info().run_id
        initialize_logger(workflow_id, run_id)

        results = []
        parallel_tasks = []

        workflow.logger.info("Hello from python scripts executor")
        workflow.logger.info("Starting scripts: %s" % (script_path))
        
        if os.path.isfile(script_path) and script_path.endswith(".py"):
            parent_dir = os.path.basename(os.path.dirname(script_path))
            base_name = os.path.splitext(os.path.basename(script_path))[0]
            activity_name = f"{parent_dir}/{base_name}"
            self._state.script_path = script_path
            self._state.input_data = input_data
            result = await self._execute_activity_with_state(
                activity_name, script_path, input_data
            )
            return [result]
        
        for item in sorted(os.listdir(script_path)):
            full_path = os.path.join(script_path, item)
            
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
                    args=[os.path.join(os.path.dirname(script_path), workflow_name), input_data],
                    id=child_workflow_id
                )
                
                results.append(child_result)
        
        if parallel_tasks:
            parallel_results = await asyncio.gather(*parallel_tasks)
            results.extend(parallel_results)
        
        return results