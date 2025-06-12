"""
OLAP table definitions for Moose Data Model v2 (dmv2).

This module provides classes for defining and configuring OLAP tables,
particularly for ClickHouse.
"""
from typing import Optional, Dict, Any, Generic
from pydantic import BaseModel

from moose_lib import ClickHouseEngines
from .types import TypedMooseResource, T
from ._registry import _tables

class OlapConfig(BaseModel):
    """Configuration for OLAP tables (e.g., ClickHouse tables).

    Attributes:
        order_by_fields: List of column names to use for the ORDER BY clause.
                       Crucial for `ReplacingMergeTree` and performance.
        deduplicate: If True, uses the ReplacingMergeTree engine for automatic
                     deduplication based on `order_by_fields`. Equivalent to
                     setting `engine=ClickHouseEngines.ReplacingMergeTree`.
        engine: The ClickHouse table engine to use (e.g., MergeTree, ReplacingMergeTree).
        version: Optional version string for tracking configuration changes.
        metadata: Optional metadata for the table.
    """
    order_by_fields: list[str] = []
    # equivalent to setting `engine=ClickHouseEngines.ReplacingMergeTree`
    deduplicate: bool = False
    engine: Optional[ClickHouseEngines] = None
    version: Optional[str] = None
    metadata: Optional[dict] = None

class OlapTable(TypedMooseResource, Generic[T]):
    """Represents an OLAP table (e.g., a ClickHouse table) typed with a Pydantic model.

    Args:
        name: The name of the OLAP table.
        config: Configuration options for the table engine, ordering, etc.
        t: The Pydantic model defining the table schema (passed via `OlapTable[MyModel](...)`).

    Attributes:
        config (OlapConfig): The configuration settings for this table.
        columns (Columns[T]): Helper for accessing column names safely.
        name (str): The name of the table.
        model_type (type[T]): The Pydantic model associated with this table.
        kind: The kind of the table (e.g., "OlapTable").
    """
    config: OlapConfig
    kind: str = "OlapTable"

    def __init__(self, name: str, config: OlapConfig = OlapConfig(), **kwargs):
        super().__init__()
        self._set_type(name, self._get_type(kwargs))
        self.config = config
        self.metadata = config.metadata
        _tables[name] = self