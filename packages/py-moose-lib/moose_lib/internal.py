"""
Internal utilities for Moose Python library.

This module contains Pydantic models representing the configuration signature
of various Moose resources (tables, streams/topics, APIs) and functions
to convert the user-defined resources (from `dmv2.py`) into a serializable
JSON format expected by the Moose infrastructure management system.
"""
from importlib import import_module
from typing import Literal, Optional, List, Any, Dict
from pydantic import BaseModel, ConfigDict, AliasGenerator, Field
import json
from .data_models import Column, _to_columns
from moose_lib.dmv2 import (
    get_tables,
    get_streams,
    get_ingest_apis,
    get_apis,
    get_sql_resources,
    get_workflows,
    OlapTable,
    View,
    MaterializedView,
    SqlResource
)
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
        metadata: Optional metadata for the target stream.
    """
    kind: Literal["stream"]
    name: str
    version: Optional[str] = None
    metadata: Optional[dict] = None

class Consumer(BaseModel):
    """Represents a consumer attached to a stream.

    Attributes:
        version: Optional version of the consumer configuration.
    """
    version: Optional[str] = None

class EngineConfigDict(BaseModel):
    """Engine configuration using discriminated union pattern for serialization."""
    engine: str
    # S3Queue-specific fields (only present when engine is "S3Queue")
    s3_path: Optional[str] = None
    format: Optional[str] = None
    aws_access_key_id: Optional[str] = None
    aws_secret_access_key: Optional[str] = None
    compression: Optional[str] = None
    headers: Optional[Dict[str, str]] = None
    s3_settings: Optional[Dict[str, Any]] = None

class TableConfig(BaseModel):
    """Internal representation of an OLAP table configuration for serialization.

    Attributes:
        name: Name of the table.
        columns: List of columns with their types and attributes.
        order_by: List of columns used for the ORDER BY clause.
        engine_config: Engine configuration with type-safe, engine-specific parameters.
        version: Optional version string of the table configuration.
        metadata: Optional metadata for the table.
        life_cycle: Lifecycle management setting for the table.
    """
    model_config = model_config

    name: str
    columns: List[Column]
    order_by: List[str]
    engine_config: Optional[EngineConfigDict] = None
    version: Optional[str] = None
    metadata: Optional[dict] = None
    life_cycle: Optional[str] = None

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
        metadata: Optional metadata for the topic.
        life_cycle: Lifecycle management setting for the topic.
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
    metadata: Optional[dict] = None
    life_cycle: Optional[str] = None

class IngestApiConfig(BaseModel):
    """Internal representation of an Ingest API configuration for serialization.

    Attributes:
        name: Name of the Ingest API.
        columns: List of columns expected in the input data.
        write_to: The target stream where the ingested data is written.
        version: Optional version string of the API configuration.
        metadata: Optional metadata for the API.
    """
    model_config = model_config

    name: str
    columns: List[Column]
    write_to: Target
    dead_letter_queue: Optional[str] = None
    version: Optional[str] = None
    metadata: Optional[dict] = None

class InternalApiConfig(BaseModel):
    """Internal representation of a API configuration for serialization.

    Attributes:
        name: Name of the API.
        query_params: List of columns representing the expected query parameters.
        response_schema: JSON schema definition of the API's response body.
        version: Optional version string of the API configuration.
        metadata: Optional metadata for the API.
    """
    model_config = model_config

    name: str
    query_params: List[Column]
    response_schema: JsonSchemaValue
    version: Optional[str] = None
    metadata: Optional[dict] = None

class WorkflowJson(BaseModel):
    """Internal representation of a workflow configuration for serialization.

    Attributes:
        name: Name of the workflow.
        retries: Optional number of retry attempts for the entire workflow.
        timeout: Optional timeout string for the entire workflow.
        schedule: Optional cron-like schedule string for recurring execution.
    """
    model_config = model_config

    name: str
    retries: Optional[int] = None
    timeout: Optional[str] = None
    schedule: Optional[str] = None

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
        metadata: Optional metadata for the resource.
    """
    model_config = model_config

    name: str
    setup: list[str]
    teardown: list[str]
    pulls_data_from: list[InfrastructureSignatureJson]
    pushes_data_to: list[InfrastructureSignatureJson]
    metadata: Optional[dict] = None


class InfrastructureMap(BaseModel):
    """Top-level model holding the configuration for all defined Moose resources.

    This structure is serialized to JSON and passed to the Moose infrastructure system.

    Attributes:
        tables: Dictionary mapping table names to their configurations.
        topics: Dictionary mapping topic/stream names to their configurations.
        ingest_apis: Dictionary mapping ingest API names to their configurations.
        apis: Dictionary mapping API names to their configurations.
        sql_resources: Dictionary mapping SQL resource names to their configurations.
        workflows: Dictionary mapping workflow names to their configurations.
    """
    model_config = model_config

    tables: dict[str, TableConfig]
    topics: dict[str, TopicConfig]
    ingest_apis: dict[str, IngestApiConfig]
    apis: dict[str, InternalApiConfig]
    sql_resources: dict[str, SqlResourceConfig]
    workflows: dict[str, WorkflowJson]


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


def _convert_engine_to_config_dict(engine_enum, table) -> EngineConfigDict:
    """Convert engine enum and table configuration to new engine config format.
    
    Args:
        engine_enum: The ClickHouseEngines enum value
        table: The OlapTable instance with configuration
        
    Returns:
        EngineConfigDict with engine-specific configuration
    """
    from moose_lib import ClickHouseEngines
    from moose_lib.blocks import S3QueueEngine
    
    # Check if the table uses the new engine configuration classes
    if hasattr(table.config, 'engine') and hasattr(table.config.engine, '__class__'):
        engine_instance = table.config.engine
        if isinstance(engine_instance, S3QueueEngine):
            return EngineConfigDict(
                engine="S3Queue",
                s3_path=engine_instance.s3_path,
                format=engine_instance.format,
                aws_access_key_id=engine_instance.aws_access_key_id,
                aws_secret_access_key=engine_instance.aws_secret_access_key,
                compression=engine_instance.compression,
                headers=engine_instance.headers,
                s3_settings=engine_instance.s3_settings
            )
    
    # Handle legacy enum-based engine configuration
    engine_name = engine_enum.value if hasattr(engine_enum, 'value') else str(engine_enum)
    
    # For S3Queue with legacy configuration, check for s3_queue_engine_config
    if engine_name == "S3Queue" and hasattr(table.config, 's3_queue_engine_config'):
        s3_config = table.config.s3_queue_engine_config
        if s3_config:
            return EngineConfigDict(
                engine="S3Queue",
                s3_path=s3_config.path,
                format=s3_config.format,
                aws_access_key_id=s3_config.aws_access_key_id,
                aws_secret_access_key=s3_config.aws_secret_access_key,
                compression=s3_config.compression,
                headers=s3_config.headers,
                s3_settings=s3_config.settings
            )
    
    # For all other engines, just return the engine name
    return EngineConfigDict(
        engine=engine_name,
        s3_path=None,
        aws_access_key_id=None,
        aws_secret_access_key=None,
        s3_settings=None
    )

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
    apis = {}
    sql_resources = {}
    workflows = {}

    for name, table in get_tables().items():
        # Convert engine configuration to new format
        engine_config = None
        if table.config.engine:
            engine_config = _convert_engine_to_config_dict(table.config.engine, table)
        
        tables[name] = TableConfig(
            name=name,
            columns=_to_columns(table._t),
            order_by=table.config.order_by_fields,
            engine_config=engine_config,
            version=table.config.version,
            metadata=getattr(table, "metadata", None),
            life_cycle=table.config.life_cycle.value if table.config.life_cycle else None,
        )

    for name, stream in get_streams().items():
        transformation_targets = [
            Target(
                kind="stream",
                name=dest_name,
                version=transform.config.version,
                metadata=getattr(transform.config, "metadata", None),
            )
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
            consumers=consumers,
            metadata=getattr(stream, "metadata", None),
            life_cycle=stream.config.life_cycle.value if stream.config.life_cycle else None,
        )

    for name, api in get_ingest_apis().items():
        ingest_apis[name] = IngestApiConfig(
            name=name,
            columns=_to_columns(api._t),
            version=api.config.version,
            write_to=Target(
                kind="stream",
                name=api.config.destination.name
            ),
            metadata=getattr(api, "metadata", None),
            dead_letter_queue=api.config.dead_letter_queue.name if api.config.dead_letter_queue else None
        )

    for name, api in get_apis().items():
        apis[name] = InternalApiConfig(
            name=api.name,
            query_params=_to_columns(api.model_type),
            response_schema=api.get_response_schema(),
            version=api.config.version,
            metadata=getattr(api, "metadata", None),
        )

    for name, resource in get_sql_resources().items():
        sql_resources[name] = SqlResourceConfig(
            name=resource.name,
            setup=resource.setup,
            teardown=resource.teardown,
            pulls_data_from=[_map_sql_resource_ref(dep) for dep in resource.pulls_data_from],
            pushes_data_to=[_map_sql_resource_ref(dep) for dep in resource.pushes_data_to],
            metadata=getattr(resource, "metadata", None),
        )

    for name, workflow in get_workflows().items():
        workflows[name] = WorkflowJson(
            name=workflow.name,
            retries=workflow.config.retries,
            timeout=workflow.config.timeout,
            schedule=workflow.config.schedule,
        )

    infra_map = InfrastructureMap(
        tables=tables,
        topics=topics,
        ingest_apis=ingest_apis,
        apis=apis,
        sql_resources=sql_resources,
        workflows=workflows
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
