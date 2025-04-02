from importlib import import_module
from typing import Literal, Optional, List, Any
from pydantic import BaseModel, ConfigDict, AliasGenerator
import json
from .data_models import Column, _to_columns
from moose_lib.dmv2 import _tables, _streams, _ingest_apis, _egress_apis, SqlResource, _sql_resources
from pydantic.alias_generators import to_camel
from pydantic.json_schema import JsonSchemaValue

model_config = ConfigDict(alias_generator=AliasGenerator(
    serialization_alias=to_camel,
))


class Target(BaseModel):
    kind: Literal["stream"]
    name: str


class TableConfig(BaseModel):
    model_config = model_config

    name: str
    columns: List[Column]
    order_by: List[str]
    deduplicate: bool
    engine: Optional[str]


class TopicConfig(BaseModel):
    model_config = model_config

    name: str
    columns: List[Column]
    target_table: Optional[str] = None
    retention_period: int
    partition_count: int
    transformation_targets: List[Target]
    has_multi_transform: bool


class IngestApiConfig(BaseModel):
    model_config = model_config

    name: str
    columns: List[Column]
    format: str
    write_to: Target


class EgressApiConfig(BaseModel):
    model_config = model_config

    name: str
    query_params: List[Column]
    response_schema: JsonSchemaValue


class SqlResourceConfig(BaseModel):
    model_config = model_config

    name: str
    setup: list[str]
    teardown: list[str]


class InfrastructureMap(BaseModel):
    model_config = model_config

    tables: dict[str, TableConfig]
    topics: dict[str, TopicConfig]
    ingest_apis: dict[str, IngestApiConfig]
    egress_apis: dict[str, EgressApiConfig]
    sql_resources: dict[str, SqlResourceConfig]


def to_infra_map() -> dict:
    """
    Converts the internal registries to a structured infrastructure map format.
    
    Returns:
        A dictionary with tables, topics (streams), and ingest APIs.
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
            engine=None if engine is None else engine.value
        )

    for name, stream in _streams.items():
        transformation_targets = [
            Target(kind="stream", name=dest_name)
            for dest_name, (destination, _) in stream.transformations.items()
        ]

        topics[name] = TopicConfig(
            name=name,
            columns=_to_columns(stream._t),
            target_table=stream.config.destination.name if stream.config.destination else None,
            retention_period=stream.config.retention_period,
            partition_count=stream.config.parallelism,
            transformation_targets=transformation_targets,
            has_multi_transform=stream._multipleTransformations is not None
        )

    for name, api in _ingest_apis.items():
        ingest_apis[name] = IngestApiConfig(
            name=name,
            columns=_to_columns(api._t),
            format=api.config.format.value,
            write_to=Target(
                kind="stream",
                name=api.config.destination.name
            )
        )

    for name, api in _egress_apis.items():
        egress_apis[name] = EgressApiConfig(
            name=name,
            query_params=_to_columns(api.model_type),
            response_schema=api.get_response_schema()
        )

    for name, resource in _sql_resources.items():
        sql_resources[name] = SqlResourceConfig(
            name=resource.name,
            setup=resource.setup,
            teardown=resource.teardown
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
    """
    Loads the data models from a app/main.py and prints the infrastructure configuration.
    """
    import_module("app.main")

    # Generate the infrastructure map
    infra_map = to_infra_map()

    # Print in the format expected by the infrastructure system
    print("___MOOSE_STUFF___start", json.dumps(infra_map), "end___MOOSE_STUFF___")
