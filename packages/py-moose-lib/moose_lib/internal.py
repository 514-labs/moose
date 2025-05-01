"""
Internal utilities for Moose Python library.

This module contains Pydantic models representing the configuration signature
of various Moose resources (tables, streams/topics, APIs) and functions
to convert the user-defined resources (from `dmv2.py`) into a serializable
JSON format expected by the Moose infrastructure management system.
"""
from importlib import import_module
from typing import Literal, Optional, List, Any
from pydantic import BaseModel, ConfigDict, AliasGenerator
import json
from .data_models import Column, _to_columns
from moose_lib.dmv2 import _tables, _streams, _ingest_apis, _egress_apis, SqlResource, _sql_resources
from moose_lib.dmv2 import OlapTable, View, MaterializedView
from pydantic.alias_generators import to_camel
from pydantic.json_schema import JsonSchemaValue

model_config = ConfigDict(alias_generator=AliasGenerator(
    serialization_alias=to_camel,
))


class Target(BaseModel):
    """Represents a target destination for data flow, typically a stream.

    Attributes:
        kind: The type of the target (currently only "stream").
        name: The name of the target stream.
        version: Optional version of the target stream configuration.
    """
    kind: Literal["stream"]
    name: str
    version: Optional[str] = None

class Consumer(BaseModel):
    """Represents a consumer attached to a stream.

    Attributes:
        version: Optional version of the consumer configuration.
    """
    version: Optional[str] = None

class TableConfig(BaseModel):
    """Internal representation of an OLAP table configuration for serialization.

    Attributes:
        name: Name of the table.
        columns: List of columns with their types and attributes.
        order_by: List of columns used for the ORDER BY clause.
        deduplicate: Whether the table uses a deduplicating engine (e.g., ReplacingMergeTree).
        engine: The name of the ClickHouse engine used.
        version: Optional version string of the table configuration.
    """
    model_config = model_config

    name: str
    columns: List[Column]
    order_by: List[str]
    deduplicate: bool
    engine: Optional[str]
    version: Optional[str] = None

class TopicConfig(BaseModel):
    """Internal representation of a stream/topic configuration for serialization.

    Attributes:
        name: Name of the topic.
        columns: List of columns (fields) in the topic messages.
        target_table: Optional name of the OLAP table this topic automatically syncs to.
        target_table_version: Optional version of the target table configuration.
        version: Optional version string of the topic configuration.
        retention_period: Data retention period in seconds.
        partition_count: Number of partitions.
        transformation_targets: List of streams this topic transforms data into.
        has_multi_transform: Flag indicating if a multi-transform function is defined.
        consumers: List of consumers attached to this topic.
    """
    model_config = model_config

    name: str
    columns: List[Column]
    target_table: Optional[str] = None
    target_table_version: Optional[str] = None
    version: Optional[str] = None
    retention_period: int
    partition_count: int
    transformation_targets: List[Target]
    has_multi_transform: bool
    consumers: List[Consumer]

class IngestApiConfig(BaseModel):
    """Internal representation of an Ingest API configuration for serialization.

    Attributes:
        name: Name of the Ingest API.
        columns: List of columns expected in the input data.
        format: The expected input data format (e.g., "JSON").
        write_to: The target stream where the ingested data is written.
        version: Optional version string of the API configuration.
    """
    model_config = model_config

    name: str
    columns: List[Column]
    format: str
    write_to: Target
    version: Optional[str] = None

class EgressApiConfig(BaseModel):
    """Internal representation of a Consumption (Egress) API configuration for serialization.

    Attributes:
        name: Name of the Egress API.
        query_params: List of columns representing the expected query parameters.
        response_schema: JSON schema definition of the API's response body.
        version: Optional version string of the API configuration.
    """
    model_config = model_config

    name: str
    query_params: List[Column]
    response_schema: JsonSchemaValue
    version: Optional[str] = None

class InfrastructureSignatureJson(BaseModel):
    """Represents the unique signature of an infrastructure component (Table, Topic, etc.).

    Used primarily for defining dependencies between SQL resources.

    Attributes:
        id: A unique identifier for the resource instance (often name + version).
        kind: The type of the infrastructure component.
    """
    id: str
    kind: Literal["Table", "Topic", "ApiEndpoint", "TopicToTableSyncProcess", "View", "SqlResource"]

class SqlResourceConfig(BaseModel):
    """Internal representation of a generic SQL resource (like View, MaterializedView) for serialization.

    Attributes:
        name: Name of the SQL resource.
        setup: List of SQL commands required to create the resource.
        teardown: List of SQL commands required to drop the resource.
        pulls_data_from: List of infrastructure components this resource reads from.
        pushes_data_to: List of infrastructure components this resource writes to.
    """
    model_config = model_config

    name: str
    setup: list[str]
    teardown: list[str]
    pulls_data_from: list[InfrastructureSignatureJson]
    pushes_data_to: list[InfrastructureSignatureJson]


class InfrastructureMap(BaseModel):
    """Top-level model holding the configuration for all defined Moose resources.

    This structure is serialized to JSON and passed to the Moose infrastructure system.

    Attributes:
        tables: Dictionary mapping table names to their configurations.
        topics: Dictionary mapping topic/stream names to their configurations.
        ingest_apis: Dictionary mapping ingest API names to their configurations.
        egress_apis: Dictionary mapping egress API names to their configurations.
        sql_resources: Dictionary mapping SQL resource names to their configurations.
    """
    model_config = model_config

    tables: dict[str, TableConfig]
    topics: dict[str, TopicConfig]
    ingest_apis: dict[str, IngestApiConfig]
    egress_apis: dict[str, EgressApiConfig]
    sql_resources: dict[str, SqlResourceConfig]


def _map_sql_resource_ref(r: Any) -> InfrastructureSignatureJson:
    """Maps a `dmv2` SQL resource object to its `InfrastructureSignatureJson`.

    Determines the correct `kind` and generates the `id` based on the resource
    type and its configuration (e.g., including version if present).

    Args:
        r: An instance of OlapTable, View, MaterializedView, or SqlResource.

    Returns:
        An InfrastructureSignatureJson representing the resource.

    Raises:
        TypeError: If the input object is not a recognized SQL resource type.
    """
    if hasattr(r, 'kind'):
        if r.kind == "OlapTable":
            # Explicitly cast for type hint checking if needed, though Python is dynamic
            table = r # type: OlapTable
            res_id = f"{table.name}_{table.config.version}" if table.config.version else table.name
            return InfrastructureSignatureJson(id=res_id, kind="Table")
        elif r.kind == "SqlResource":
            # Explicitly cast for type hint checking if needed
            resource = r # type: SqlResource
            return InfrastructureSignatureJson(id=resource.name, kind="SqlResource")
        else:
            raise TypeError(f"Unknown SQL resource kind: {r.kind} for object: {r}")
    else:
        # Fallback or error if 'kind' attribute is missing
        raise TypeError(f"Object {r} lacks a 'kind' attribute for dependency mapping.")


def to_infra_map() -> dict:
    """Converts the registered `dmv2` resources into the serializable `InfrastructureMap` format.

    Iterates through the internal registries (`_tables`, `_streams`, etc.) populated
    by the user's definitions in `app/main.py` (or elsewhere) and transforms them
    into the corresponding `*Config` Pydantic models.

    Returns:
        A dictionary representing the `InfrastructureMap`, ready for JSON serialization
        using Pydantic's `model_dump` with camelCase aliases.
    """
    tables = {}
    topics = {}
    ingest_apis = {}
    egress_apis = {}
    sql_resources = {}

    for name, table in _tables.items():
        engine = table.config.engine
        tables[name] = TableConfig(
            name=name,
            columns=_to_columns(table._t),
            order_by=table.config.order_by_fields,
            deduplicate=table.config.deduplicate,
            engine=None if engine is None else engine.value,
            version=table.config.version
        )

    for name, stream in _streams.items():
        transformation_targets = [
            Target(kind="stream", name=dest_name, version=transform.config.version)
            for dest_name, transforms in stream.transformations.items()
            for transform in transforms
        ]

        consumers = [
            Consumer(version=consumer.config.version)
            for consumer in stream.consumers
        ]

        topics[name] = TopicConfig(
            name=name,
            columns=_to_columns(stream._t),
            target_table=stream.config.destination.name if stream.config.destination else None,
            target_table_version=stream.config.destination.config.version if stream.config.destination else None,
            retention_period=stream.config.retention_period,
            partition_count=stream.config.parallelism,
            version=stream.config.version,
            transformation_targets=transformation_targets,
            has_multi_transform=stream._multipleTransformations is not None,
            consumers=consumers
        )

    for name, api in _ingest_apis.items():
        ingest_apis[name] = IngestApiConfig(
            name=name,
            columns=_to_columns(api._t),
            format=api.config.format.value,
            version=api.config.version,
            write_to=Target(
                kind="stream",
                name=api.config.destination.name
            )
        )

    for name, api in _egress_apis.items():
        egress_apis[name] = EgressApiConfig(
            name=name,
            query_params=_to_columns(api.model_type),
            response_schema=api.get_response_schema(),
            version=api.config.version
        )

    for name, resource in _sql_resources.items():
        sql_resources[name] = SqlResourceConfig(
            name=resource.name,
            setup=resource.setup,
            teardown=resource.teardown,
            pulls_data_from=[_map_sql_resource_ref(dep) for dep in resource.pulls_data_from],
            pushes_data_to=[_map_sql_resource_ref(dep) for dep in resource.pushes_data_to]
        )

    infra_map = InfrastructureMap(
        tables=tables,
        topics=topics,
        ingest_apis=ingest_apis,
        egress_apis=egress_apis,
        sql_resources=sql_resources
    )

    return infra_map.model_dump(by_alias=True)


def load_models():
    """Imports the user's main application module and prints the infrastructure map.

    This function is typically the entry point for the Moose infrastructure system
    when processing Python-defined resources.

    1. Imports `app.main`, which should trigger the registration of all Moose
       resources defined therein (OlapTable[...](...), Stream[...](...), etc.).
    2. Calls `to_infra_map()` to generate the infrastructure configuration dictionary.
    3. Prints the dictionary as a JSON string, wrapped in specific delimiters
       (`___MOOSE_STUFF___start` and `end___MOOSE_STUFF___`), which the
       calling system uses to extract the configuration.
    """
    import_module("app.main")

    # Generate the infrastructure map
    infra_map = to_infra_map()

    # Print in the format expected by the infrastructure system
    print("___MOOSE_STUFF___start", json.dumps(infra_map), "end___MOOSE_STUFF___")
