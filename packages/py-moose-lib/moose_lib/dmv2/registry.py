"""
Global registries for Moose Data Model v2 (dmv2) resources.

This module provides functions to access the registered resources.
The actual registry dictionaries are maintained in _registry.py to avoid circular dependencies.
"""
from typing import Optional, Dict
from .olap_table import OlapTable
from .stream import Stream
from .ingest_api import IngestApi
from .consumption import Api
from .sql_resource import SqlResource
from .workflow import Workflow
from ._registry import (
    _tables,
    _streams,
    _ingest_apis,
    _apis,
    _sql_resources,
    _workflows,
    _api_name_aliases,
)

def get_tables() -> Dict[str, OlapTable]:
    """Get all registered OLAP tables."""
    return _tables

def get_table(name: str) -> Optional[OlapTable]:
    """Get a registered OLAP table by name."""
    return _tables.get(name)

def get_streams() -> Dict[str, Stream]:
    """Get all registered streams."""
    return _streams

def get_stream(name: str) -> Optional[Stream]:
    """Get a registered stream by name."""
    return _streams.get(name)

def get_ingest_apis() -> Dict[str, IngestApi]:
    """Get all registered ingestion APIs."""
    return _ingest_apis

def get_ingest_api(name: str) -> Optional[IngestApi]:
    """Get a registered ingestion API by name."""
    return _ingest_apis.get(name)

def get_apis() -> Dict[str, Api]:
    """Get all registered APIs."""
    return _apis

def get_api(name: str) -> Optional[Api]:
    """Get a registered API by name.

    Supports unversioned lookup by name via alias map when only a single versioned API exists.
    """
    return _apis.get(name) or _api_name_aliases.get(name)

def get_sql_resources() -> Dict[str, SqlResource]:
    """Get all registered SQL resources."""
    return _sql_resources

def get_sql_resource(name: str) -> Optional[SqlResource]:
    """Get a registered SQL resource by name."""
    return _sql_resources.get(name)

def get_workflows() -> Dict[str, Workflow]:
    """Get all registered workflows."""
    return _workflows

def get_workflow(name: str) -> Optional[Workflow]:
    """Get a registered workflow by name."""
    return _workflows.get(name)


# Backward compatibility aliases (deprecated)
get_consumption_apis = get_apis
"""@deprecated: Use get_apis instead of get_consumption_apis"""

get_consumption_api = get_api
"""@deprecated: Use get_api instead of get_consumption_api"""