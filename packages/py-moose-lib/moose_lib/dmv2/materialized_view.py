"""
Materialized View definitions for Moose Data Model v2 (dmv2).

This module provides classes for defining Materialized Views,
including their SQL statements, target tables, and dependencies.
"""
from typing import Any, Optional, Union, Generic, TypeVar
from pydantic import BaseModel, ConfigDict

from moose_lib import ClickHouseEngines
from .types import BaseTypedResource, T
from .olap_table import OlapTable, OlapConfig
from .sql_resource import SqlResource

class MaterializedViewOptions(BaseModel):
    """Configuration options for creating a Materialized View.

    Attributes:
        select_statement: The SQL SELECT statement defining the view's data.
        select_tables: List of source tables/views the select statement reads from.
        table_name: The name of the underlying target table storing the materialized data.
        materialized_view_name: The name of the MATERIALIZED VIEW object itself.
        engine: Optional ClickHouse engine for the target table.
        order_by_fields: Optional ordering key for the target table (required for
                         engines like ReplacingMergeTree).
        model_config: ConfigDict for Pydantic validation
    """
    select_statement: str
    select_tables: list[Union[OlapTable, SqlResource]]
    table_name: str
    materialized_view_name: str
    engine: Optional[ClickHouseEngines] = None
    order_by_fields: Optional[list[str]] = None
    metadata: Optional[dict] = None
    # Ensure arbitrary types are allowed for Pydantic validation
    model_config = ConfigDict(arbitrary_types_allowed=True)

class MaterializedView(SqlResource, BaseTypedResource, Generic[T]):
    """Represents a ClickHouse Materialized View.

    Encapsulates the MATERIALIZED VIEW definition and the underlying target `OlapTable`
    that stores the data.

    Args:
        options: Configuration defining the select statement, names, and dependencies.
        t: The Pydantic model defining the schema of the target table
           (passed via `MaterializedView[MyModel](...)`).

    Attributes:
        target_table (OlapTable[T]): The `OlapTable` instance storing the materialized data.
        config (MaterializedViewOptions): The configuration options used to create the view.
        name (str): The name of the MATERIALIZED VIEW object.
        model_type (type[T]): The Pydantic model associated with the target table.
        setup (list[str]): SQL commands to create the view and populate the target table.
        teardown (list[str]): SQL command to drop the view.
        pulls_data_from (list[SqlObject]): Source tables/views.
        pushes_data_to (list[SqlObject]): The target table.
    """
    target_table: OlapTable[T]
    config: MaterializedViewOptions

    def __init__(
            self,
            options: MaterializedViewOptions,
            **kwargs
    ):
        self._set_type(options.materialized_view_name, self._get_type(kwargs))

        setup = [
            f"CREATE MATERIALIZED VIEW IF NOT EXISTS {options.materialized_view_name} TO {options.table_name} AS {options.select_statement}",
            f"INSERT INTO {options.table_name} {options.select_statement}"
        ]
        teardown = [f"DROP VIEW IF EXISTS {options.materialized_view_name}"]

        target_table = OlapTable(
            name=options.table_name,
            config=OlapConfig(
                order_by_fields=options.order_by_fields or [],
                engine=options.engine
            ),
            t=self._t
        )

        super().__init__(
            options.materialized_view_name,
            setup,
            teardown,
            pulls_data_from=options.select_tables,
            pushes_data_to=[target_table],
            metadata=options.metadata
        )

        self.target_table = target_table
        self.config = options