from typing import Literal, Optional, List, Any
from pydantic import BaseModel, ConfigDict

from .data_models import Column, _to_columns
from moose_lib.dmv2 import _tables, _streams, _ingest_apis
from pydantic.alias_generators import to_camel

model_config = ConfigDict(alias_generator=to_camel)


class Target(BaseModel):
    model_config = model_config
    kind: Literal["stream"]
    name: str


class TableConfig(BaseModel):
    name: str
    columns: List[Column]
    order_by: List[str]
    deduplicate: bool


class TopicConfig(BaseModel):
    name: str
    columns: List[Column]
    target_table: Optional[str] = None
    retention_period: int
    partition_count: int
    transformation_targets: List[Target]
    has_multi_transform: bool


class IngestApiConfig(BaseModel):
    name: str
    columns: List[Column]
    format: str
    write_to: Target


class InfrastructureMap(BaseModel):
    tables: dict[str, TableConfig]
    topics: dict[str, TopicConfig]
    ingest_apis: dict[str, IngestApiConfig]


def to_infra_map() -> dict:
    """
    Converts the internal registries to a structured infrastructure map format.
    
    Returns:
        A dictionary with tables, topics (streams), and ingest APIs.
    """
    tables = {}
    topics = {}
    ingest_apis = {}

    for name, table in _tables.items():
        tables[name] = TableConfig(
            name=name,
            columns=_to_columns(table._t),
            order_by=table.config.order_by_fields,
            deduplicate=table.config.deduplicate
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

    infra_map = InfrastructureMap(
        tables=tables,
        topics=topics,
        ingest_apis=ingest_apis
    )

    return infra_map.model_dump(by_alias=True)


def load_models(module_path: str = None) -> dict:
    """
    Loads the data models from a specified module and returns the infrastructure configuration.
    
    Args:
        module_path: Path to the module to load (can be a file path or module name).
                    If None, looks for 'main.py' in the current directory.
    
    Returns:
        A dictionary with the infrastructure configuration.
    """
    import importlib
    import importlib.util
    import os
    import sys
    import json

    if module_path is None:
        # Try common model definition filenames, starting with main.py
        possible_paths = [
            os.path.join(os.getcwd(), "main.py"),
            os.path.join(os.getcwd(), "app", "main.py"),
            os.path.join(os.getcwd(), "models.py"),
            os.path.join(os.getcwd(), "data_models.py"),
            os.path.join(os.getcwd(), "app", "models.py")
        ]

        for path in possible_paths:
            if os.path.exists(path):
                module_path = path
                break

        if module_path is None:
            raise FileNotFoundError(
                "Could not find model definitions. Please specify the module path."
            )

    # Check if it's a file path or module name
    if os.path.exists(module_path):
        # Load from file path
        dir_path = os.path.dirname(os.path.abspath(module_path))
        if dir_path not in sys.path:
            sys.path.insert(0, dir_path)

        module_name = os.path.basename(module_path).replace(".py", "")
        spec = importlib.util.spec_from_file_location(module_name, module_path)
        if spec and spec.loader:
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
    else:
        # Try to import as a module name
        try:
            importlib.import_module(module_path)
        except ImportError:
            raise ImportError(f"Could not load module: {module_path}")

    # Generate the infrastructure map
    infra_map = to_infra_map()

    # Print in the format expected by the infrastructure system
    print("___MOOSE_STUFF___start", infra_map, "end___MOOSE_STUFF___")
