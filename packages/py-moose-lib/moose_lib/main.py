"""Core Moose Python library definitions.

This module provides foundational classes, enums, and functions used across the Moose ecosystem,
including configuration objects, clients for interacting with services (ClickHouse, Temporal),
and utilities for defining data models and SQL queries.
"""
from clickhouse_connect.driver.client import Client as ClickhouseClient
from temporalio.client import Client as TemporalClient
from dataclasses import dataclass, asdict
from enum import Enum
from typing import Any, Callable, Dict, Optional, TypeVar, overload
import sys
import os
import json


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
class ConsumptionApiResult:
    """Standard structure for returning results from a Consumption API handler.

    Attributes:
        status: The HTTP status code for the response.
        body: The response body, which should be JSON serializable.
    """
    status: int
    body: Any


class QueryClient:
    """Client for executing queries, typically against ClickHouse.

    (Note: Current implementation is a placeholder.)

    Args:
        ch_client: An instance of the ClickHouse client.
    """
    def __init__(self, ch_client: ClickhouseClient):
        self.ch_client = ch_client

    def execute(self, input, variables) -> Any:
        # No impl for the interface
        pass


class WorkflowClient:
    """Client for interacting with Temporal workflows.

    (Note: Current implementation is a placeholder.)

    Args:
        temporal_client: An instance of the Temporal client.
    """
    def __init__(self, temporal_client: TemporalClient):
        self.temporal_client = temporal_client

    def execute(self, name: str, input_data: Any) -> Dict[str, Any]:
        # No impl for the interface
        pass


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
        if temporal_client:
            self.workflow = WorkflowClient(temporal_client)
        else:
            self.workflow = None


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

