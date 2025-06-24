"""
Moose Data Model v2 (dmv2)

This package provides the Python classes for defining Moose v2 data model resources.
"""

from .types import (
    BaseTypedResource,
    TypedMooseResource,
    Columns,
    T,
    U,
    T_none,
    U_none,
    ZeroOrMany,
)

from .olap_table import (
    OlapConfig,
    OlapTable,
    InsertOptions,
)

from .stream import (
    StreamConfig,
    TransformConfig,
    ConsumerConfig,
    Stream,
    DeadLetterModel,
    DeadLetterQueue,
)

from .ingest_api import (
    IngestConfig,
    IngestConfigWithDestination,
    IngestApi,
)

from .ingest_pipeline import (
    IngestPipelineConfig,
    IngestPipeline,
)

from .consumption import (
    EgressConfig,
    ConsumptionApi,
    get_moose_base_url,
    set_moose_base_url,
)

from .sql_resource import (
    SqlResource,
)

from .view import (
    View,
)

from .materialized_view import (
    MaterializedViewOptions,
    MaterializedView,
)

from .workflow import (
    TaskConfig,
    Task,
    WorkflowConfig,
    Workflow,
)

from .registry import (
    get_tables,
    get_table,
    get_streams,
    get_stream,
    get_ingest_apis,
    get_ingest_api,
    get_consumption_apis,
    get_consumption_api,
    get_sql_resources,
    get_sql_resource,
    get_workflows,
    get_workflow,
)

__all__ = [
    # Types
    'BaseTypedResource',
    'TypedMooseResource',
    'Columns',
    'T',
    'U',
    'T_none',
    'U_none',
    'ZeroOrMany',

    # OLAP Tables
    'OlapConfig',
    'OlapTable',
    'InsertOptions',

    # Streams
    'StreamConfig',
    'TransformConfig',
    'ConsumerConfig',
    'Stream',
    'DeadLetterModel',
    'DeadLetterQueue',

    # Ingestion
    'IngestConfig',
    'IngestConfigWithDestination',
    'IngestPipelineConfig',
    'IngestApi',
    'IngestPipeline',

    # Consumption
    'EgressConfig',
    'ConsumptionApi',
    'get_moose_base_url',
    'set_moose_base_url',

    # SQL
    'SqlResource',
    'View',
    'MaterializedViewOptions',
    'MaterializedView',

    # Workflow
    'TaskConfig',
    'Task',
    'WorkflowConfig',
    'Workflow',

    # Registry
    'get_tables',
    'get_table',
    'get_streams',
    'get_stream',
    'get_ingest_apis',
    'get_ingest_api',
    'get_consumption_apis',
    'get_consumption_api',
    'get_sql_resources',
    'get_sql_resource',
    'get_workflows',
    'get_workflow',
]
