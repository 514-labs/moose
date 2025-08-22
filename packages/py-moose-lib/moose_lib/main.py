"""Core Moose Python library definitions.

This module provides foundational classes, enums, and functions used across the Moose ecosystem,
including configuration objects, clients for interacting with services (ClickHouse, Temporal),
and utilities for defining data models and SQL queries.
"""
from clickhouse_connect.driver.client import Client as ClickhouseClient
from clickhouse_connect import get_client
from pydantic import BaseModel
from dataclasses import dataclass, asdict
from enum import Enum
from typing import Any, Callable, Dict, Optional, TypeVar, overload, Type, Union
import sys
import os
import json
import hashlib
import asyncio
from string import Formatter
from temporalio.client import Client as TemporalClient, TLSConfig
from temporalio.common import RetryPolicy, WorkflowIDConflictPolicy, WorkflowIDReusePolicy
from datetime import timedelta
from .config.runtime import RuntimeClickHouseConfig

from moose_lib.commons import EnhancedJSONEncoder


@dataclass
class StreamingFunction:
    """Represents a function intended for stream processing.

    Attributes:
        run: The callable function that performs the streaming logic.
    """
    run: Callable


@dataclass
class StorageConfig:
    """Configuration related to data storage, typically in an OLAP table.

    Attributes:
        enabled: Whether storage is enabled for this data model.
        order_by_fields: List of fields to use for ordering in the storage layer.
        deduplicate: Whether to enable deduplication based on the order_by_fields.
    """
    enabled: Optional[bool] = None
    order_by_fields: Optional[list[str]] = None
    deduplicate: Optional[bool] = None


@dataclass
class DataModelConfig:
    """Top-level configuration for a Moose data model.

    Combines ingestion and storage settings.

    Attributes:
        storage: Configuration for how data is stored.
    """
    storage: Optional[StorageConfig] = None


class CustomEncoder(json.JSONEncoder):
    """Custom JSON encoder that handles Enum types by encoding their values."""

    def default(self, obj):
        if isinstance(obj, Enum):
            return obj.value
        return super().default(obj)


_DC = TypeVar("_DC", bound=type)


@overload
def moose_data_model(arg: Optional[DataModelConfig]) -> Callable[[_DC], _DC]:
    """Decorator overload: Applies configuration to a data model class."""
    ...


@overload
def moose_data_model(arg: _DC) -> _DC:
    """Decorator overload: Decorates a data model class without explicit configuration."""
    ...


def moose_data_model(arg: Any = None) -> Any:
    """Decorator for Moose data model classes.

    This decorator can be used with or without arguments:
    - `@moose_data_model`: Decorates a class as a Moose data model with default settings.
    - `@moose_data_model(DataModelConfig(...))`: Decorates a class and applies the specified
      ingestion and storage configurations.

    During infrastructure processing (when `MOOSE_PYTHON_DM_DUMP` environment variable
    matches the decorated class's file path), it prints the class name and configuration
    as JSON, separated by a specific delimiter (`___DATAMODELCONFIG___`).

    Args:
        arg: Either a `DataModelConfig` instance or the class being decorated.

    Returns:
        A decorator function or the decorated class.
    """

    def get_file(t: type) -> Optional[str]:
        """Helper to get the file path of a type's definition."""
        module = sys.modules.get(t.__module__)
        if module and hasattr(module, '__file__'):
            return module.__file__
        return None

    def remove_null(d: dict) -> dict:
        """Recursively removes keys with None values from a dictionary."""
        return {key: remove_null(value) if isinstance(value, dict) else value for key, value in d.items() if
                not (value is None)}

    def decorator(data_class: type) -> type:
        expected_file_name = os.environ.get("MOOSE_PYTHON_DM_DUMP")
        if expected_file_name and expected_file_name == get_file(data_class):
            output: dict[str, str | dict] = {
                'class_name': data_class.__name__
            }
            if arg:
                output["config"] = remove_null(asdict(arg))
            output_json = json.dumps(output, cls=CustomEncoder, indent=4)
            print(output_json, "___DATAMODELCONFIG___", sep="")
        return data_class

    if isinstance(arg, type):
        return moose_data_model(None)(arg)
    return decorator


JWTPayload = Dict[str, Any]


@dataclass
class ApiResult:
    """Standard structure for returning results from a Consumption API handler.

    Attributes:
        status: The HTTP status code for the response.
        body: The response body, which should be JSON serializable.
    """
    status: int
    body: Any


# Backward compatibility alias (deprecated)
ConsumptionApiResult = ApiResult
"""@deprecated: Use ApiResult instead of ConsumptionApiResult"""


class QueryClient:
    """Client for executing queries, typically against ClickHouse.

    Args:
        ch_client_or_config: Either an instance of the ClickHouse client or a RuntimeClickHouseConfig.
    """

    def __init__(self, ch_client_or_config: Union[ClickhouseClient, RuntimeClickHouseConfig]):
        if isinstance(ch_client_or_config, RuntimeClickHouseConfig):
            # Create ClickHouse client from configuration
            config = ch_client_or_config
            interface = 'https' if config.use_ssl else 'http'
            self.ch_client = get_client(
                interface=interface,
                host=config.host,
                port=int(config.port),
                username=config.username,
                password=config.password,
                database=config.database,
            )
        else:
            # Use provided ClickHouse client directly
            self.ch_client = ch_client_or_config

    def __call__(self, input, variables):
        return self.execute(input, variables)

    def execute(self, input, variables, row_type: Type[BaseModel] = None):
        params = {}
        values = {}

        for i, (_, variable_name, _, _) in enumerate(Formatter().parse(input)):
            if variable_name:
                value = variables[variable_name]
                if isinstance(value, list) and len(value) == 1:
                    # handling passing the value of the query string dict directly to variables
                    value = value[0]

                t = 'String' if isinstance(value, str) else \
                    'Int64' if isinstance(value, int) else \
                        'Float64' if isinstance(value, float) else "String"  # unknown type

                params[variable_name] = f'{{p{i}: {t}}}'
                values[f'p{i}'] = value
        clickhouse_query = input.format_map(params)

        # We are not using the result of the ping
        # but this ensures that if the clickhouse cloud service is idle, we
        # wake it up, before we send the query.
        self.ch_client.ping()

        val = self.ch_client.query(clickhouse_query, values)

        if row_type is None:
            return list(val.named_results())
        else:
            return list(row_type(**row) for row in val.named_results())

    def close(self):
        """Close the ClickHouse client connection."""
        if self.ch_client:
            try:
                self.ch_client.close()
            except Exception as e:
                print(f"Error closing ClickHouse client: {e}")


class WorkflowClient:
    """Client for interacting with Temporal workflows.

    Args:
        temporal_client: An instance of the Temporal client.
    """

    def __init__(self, temporal_client: TemporalClient):
        self.temporal_client = temporal_client
        self.configs = self.load_consolidated_configs()
        print(f"WorkflowClient - configs: {self.configs}")

    # Test workflow executor in rust if this changes significantly
    def execute(self, name: str, input_data: Any) -> Dict[str, Any]:
        try:
            workflow_id, run_id = asyncio.run(self._start_workflow_async(name, input_data))
            print(f"WorkflowClient - started workflow: {name}")
            return {
                "status": 200,
                "body": f"Workflow started: {name}. View it in the Temporal dashboard: http://localhost:8080/namespaces/default/workflows/{workflow_id}/{run_id}/history"
            }
        except Exception as e:
            print(f"WorkflowClient - error while starting workflow: {e}")
            return {
                "status": 400,
                "body": str(e)
            }

    def terminate(self, workflow_id: str) -> Dict[str, Any]:
        try:
            asyncio.run(self._terminate_workflow_async(workflow_id))
            print(f"WorkflowClient - terminated workflow: {workflow_id}")
            return {
                "status": 200,
                "body": f"Workflow terminated: {workflow_id}"
            }
        except Exception as e:
            print(f"WorkflowClient - error while terminating workflow: {e}")
            return {
                "status": 400,
                "body": str(e)
            }

    async def _terminate_workflow_async(self, workflow_id: str):
        workflow_handle = self.temporal_client.get_workflow_handle(workflow_id)
        await workflow_handle.terminate()

    async def _start_workflow_async(self, name: str, input_data: Any):
        # Extract configuration based on workflow type
        config = self._get_workflow_config(name)

        # Process input data and generate workflow ID (common logic)
        processed_input, workflow_id = self._process_input_data(name, input_data)

        # Create retry policy and timeout (common logic)
        retry_policy = RetryPolicy(maximum_attempts=config['retry_count'])
        run_timeout = self.parse_timeout_to_timedelta(config['timeout_str'])

        print(
            f"WorkflowClient - starting {'DMv2 ' if config['is_dmv2'] else ''}workflow: {name} with retry policy: {retry_policy} and timeout: {run_timeout}")

        # Start workflow with appropriate args
        workflow_args = self._build_workflow_args(name, processed_input, config['is_dmv2'])

        workflow_handle = await self.temporal_client.start_workflow(
            "ScriptWorkflow",
            args=workflow_args,
            id=workflow_id,
            task_queue="python-script-queue",
            id_conflict_policy=WorkflowIDConflictPolicy.FAIL,
            id_reuse_policy=WorkflowIDReusePolicy.ALLOW_DUPLICATE,
            retry_policy=retry_policy,
            run_timeout=run_timeout
        )

        return workflow_id, workflow_handle.result_run_id

    def _get_workflow_config(self, name: str) -> Dict[str, Any]:
        """Extract workflow configuration from DMv2 or legacy config."""
        from moose_lib.dmv2 import get_workflow

        dmv2_workflow = get_workflow(name)
        if dmv2_workflow is not None:
            return {
                'retry_count': dmv2_workflow.config.retries or 3,
                'timeout_str': dmv2_workflow.config.timeout or "1h",
                'is_dmv2': True
            }
        else:
            config = self.configs.get(name, {})
            return {
                'retry_count': config.get('retries', 3),
                'timeout_str': config.get('timeout', "1h"),
                'is_dmv2': False
            }

    def _process_input_data(self, name: str, input_data: Any) -> tuple[Any, str]:
        """Process input data and generate workflow ID."""
        workflow_id = name
        if input_data:
            try:
                # Handle Pydantic model input for DMv2
                if isinstance(input_data, BaseModel):
                    input_data = input_data.model_dump()
                elif isinstance(input_data, str):
                    input_data = json.loads(input_data)

                # Encode with custom encoder
                input_data = json.loads(
                    json.dumps({"data": input_data}, cls=EnhancedJSONEncoder)
                )

                params_str = json.dumps(input_data, sort_keys=True)
                params_hash = hashlib.sha256(params_str.encode()).hexdigest()[:16]
                workflow_id = f"{name}-{params_hash}"
            except Exception as e:
                raise ValueError(f"Invalid input data: {e}")

        return input_data, workflow_id

    def _build_workflow_args(self, name: str, input_data: Any, is_dmv2: bool) -> list:
        """Build workflow arguments based on workflow type."""
        if is_dmv2:
            return [f"{name}", input_data]
        else:
            return [f"{os.getcwd()}/app/scripts/{name}", input_data]

    # TODO: Remove when workflows dmv1 is removed
    def load_consolidated_configs(self):
        try:
            file_path = os.path.join(os.getcwd(), ".moose", "workflow_configs.json")
            with open(file_path, 'r') as file:
                data = json.load(file)
                config_map = {config['name']: config for config in data}
                return config_map
        except Exception as e:
            print(f"Could not load configs for workflows v1: {e}")

    def parse_timeout_to_timedelta(self, timeout_str: str) -> timedelta:
        if timeout_str.endswith('h'):
            return timedelta(hours=int(timeout_str[:-1]))
        elif timeout_str.endswith('m'):
            return timedelta(minutes=int(timeout_str[:-1]))
        elif timeout_str.endswith('s'):
            return timedelta(seconds=int(timeout_str[:-1]))
        else:
            raise ValueError(f"Unsupported timeout format: {timeout_str}")


class MooseClient:
    """Unified client for interacting with Moose services (Query, Workflow).

    Provides access points for executing database queries and managing workflows.

    Args:
        ch_client: An instance of the ClickHouse client.
        temporal_client: An optional instance of the Temporal client.
                       If provided, workflow functionalities are enabled.

    Attributes:
        query (QueryClient): Client for executing queries.
        workflow (Optional[WorkflowClient]): Client for workflow operations (if configured).
    """

    def __init__(self, ch_client: ClickhouseClient, temporal_client: Optional[TemporalClient] = None):
        self.query = QueryClient(ch_client)
        self.temporal_client = temporal_client
        if temporal_client:
            self.workflow = WorkflowClient(temporal_client)
        else:
            self.workflow = None

    async def cleanup(self):
        """Cleanup resources before shutdown"""
        if self.query:
            try:
                self.query.close()
            except Exception as e:
                print(f"Error closing Clickhouse client: {e}")

        if self.temporal_client:
            try:
                await self.temporal_client.close()
            except Exception as e:
                print(f"Error closing Temporal client: {e}")


class Sql:
    """Represents a SQL query template with embedded values.

    Allows constructing SQL queries safely by separating the query string parts
    from the values to be interpolated, similar to tagged template literals
    in other languages. Supports nesting `Sql` objects.

    Args:
        raw_strings: List of string fragments forming the SQL template.
        raw_values: List of values to be interpolated between the string fragments.
                    Values can be basic types or other `Sql` instances.

    Raises:
        TypeError: If the number of strings and values doesn't match the expected
                   pattern (len(strings) == len(values) + 1).

    Attributes:
        strings (list[str]): The flattened list of string fragments.
        values (list[Any]): The flattened list of values corresponding to the gaps
                            between the strings.
    """

    def __init__(self, raw_strings: list[str], raw_values: list['RawValue']):
        if len(raw_strings) - 1 != len(raw_values):
            if len(raw_strings) == 0:
                raise TypeError("Expected at least 1 string")
            raise TypeError(f"Expected {len(raw_strings)} strings to have {len(raw_strings) - 1} values")

        values_length = sum(1 if not isinstance(value, Sql) else len(value.values) for value in raw_values)

        self.values: list['Value'] = [None] * values_length
        self.strings: list[str] = [None] * (values_length + 1)

        self.strings[0] = raw_strings[0]

        i = 0
        pos = 0
        while i < len(raw_values):
            child = raw_values[i]
            raw_string = raw_strings[i + 1]

            if isinstance(child, Sql):
                self.strings[pos] += child.strings[0]

                for child_index in range(len(child.values)):
                    self.values[pos] = child.values[child_index]
                    pos += 1
                    self.strings[pos] = child.strings[child_index + 1]

                self.strings[pos] += raw_string
            else:
                self.values[pos] = child
                pos += 1
                self.strings[pos] = raw_string

            i += 1


def sigterm_handler():
    """Handles SIGTERM signals by printing a message and exiting gracefully."""
    print("SIGTERM received")
    sys.exit(0)
