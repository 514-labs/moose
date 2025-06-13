"""
Base SQL resource definitions for Moose Data Model v2 (dmv2).

This module provides the base class for SQL resources like Views and Materialized Views,
handling common functionality like setup/teardown SQL commands and dependency tracking.
"""
from typing import Any, Optional, Union, List
from pydantic import BaseModel

from .olap_table import OlapTable
from ._registry import _sql_resources

class SqlResource:
    """Base class for SQL resources like Views and Materialized Views.

    Handles the definition of setup (CREATE) and teardown (DROP) SQL commands
    and tracks data dependencies.

    Attributes:
        name (str): The name of the SQL resource (e.g., view name).
        setup (list[str]): SQL commands to create the resource.
        teardown (list[str]): SQL commands to drop the resource.
        pulls_data_from (list[SqlObject]): List of tables/views this resource reads from.
        pushes_data_to (list[SqlObject]): List of tables/views this resource writes to.
        kind: The kind of the SQL resource (e.g., "SqlResource").
    """
    setup: list[str]
    teardown: list[str]
    name: str
    kind: str = "SqlResource"
    pulls_data_from: list[Union[OlapTable, "SqlResource"]]
    pushes_data_to: list[Union[OlapTable, "SqlResource"]]

    def __init__(
            self,
            name: str,
            setup: list[str],
            teardown: list[str],
            pulls_data_from: Optional[list[Union[OlapTable, "SqlResource"]]] = None,
            pushes_data_to: Optional[list[Union[OlapTable, "SqlResource"]]] = None,
            metadata: dict = None
    ):
        self.name = name
        self.setup = setup
        self.teardown = teardown
        self.pulls_data_from = pulls_data_from or []
        self.pushes_data_to = pushes_data_to or []
        self.metadata = metadata
        _sql_resources[name] = self