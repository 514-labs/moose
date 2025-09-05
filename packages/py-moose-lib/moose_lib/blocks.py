from dataclasses import dataclass, field
from enum import Enum
from typing import Dict, List, Optional, Any, Union
from abc import ABC
import warnings


class ClickHouseEngines(Enum):
    MergeTree = "MergeTree"
    ReplacingMergeTree = "ReplacingMergeTree"
    SummingMergeTree = "SummingMergeTree"
    AggregatingMergeTree = "AggregatingMergeTree"
    CollapsingMergeTree = "CollapsingMergeTree"
    VersionedCollapsingMergeTree = "VersionedCollapsingMergeTree"
    GraphiteMergeTree = "GraphiteMergeTree"
    S3Queue = "S3Queue"

# ==========================
# New Engine Configuration Classes
# ==========================

@dataclass
class EngineConfig(ABC):
    """Base class for engine configurations"""
    pass

@dataclass
class MergeTreeEngine(EngineConfig):
    """Configuration for MergeTree engine"""
    pass

@dataclass  
class ReplacingMergeTreeEngine(EngineConfig):
    """Configuration for ReplacingMergeTree engine (with deduplication)"""
    pass

@dataclass
class AggregatingMergeTreeEngine(EngineConfig):
    """Configuration for AggregatingMergeTree engine"""
    pass

@dataclass
class SummingMergeTreeEngine(EngineConfig):
    """Configuration for SummingMergeTree engine"""
    pass

@dataclass
class S3QueueEngine(EngineConfig):
    """Configuration for S3Queue engine with all settings inline"""
    
    # Required fields
    s3_path: str  # S3 bucket path with wildcards (e.g., 's3://bucket/prefix/*.json')
    format: str   # Data format (e.g., 'JSONEachRow', 'CSV', 'Parquet')
    
    # Optional AWS credentials
    aws_access_key_id: Optional[str] = None
    aws_secret_access_key: Optional[str] = None
    
    # Optional configuration
    compression: Optional[str] = None  # e.g., 'gzip', 'zstd'
    headers: Optional[Dict[str, str]] = None
    
    # S3Queue-specific settings
    s3_settings: Optional[Dict[str, Any]] = field(default_factory=dict)
    
    def __post_init__(self):
        """Validate required fields and set defaults"""
        if not self.s3_path:
            raise ValueError("S3Queue engine requires 's3_path'")
        if not self.format:
            raise ValueError("S3Queue engine requires 'format'")
        
        # Ensure s3_settings is initialized
        if self.s3_settings is None:
            self.s3_settings = {}
        
        # Ensure required 'mode' parameter is present (default to 'unordered')
        if "mode" not in self.s3_settings:
            self.s3_settings["mode"] = "unordered"

# ==========================
# New Table Configuration (Recommended API)
# ==========================

@dataclass
class TableConfig:
    """Modern table configuration with engine-specific settings"""
    
    # Engine configuration (required in new API)
    engine: EngineConfig
    
    # Common settings
    name: str
    columns: Dict[str, str]
    order_by: Optional[str] = None
    
    @classmethod
    def with_s3_queue(cls,
                      name: str,
                      columns: Dict[str, str],
                      s3_path: str,
                      format: str,
                      order_by: Optional[str] = None,
                      **kwargs) -> 'TableConfig':
        """Create a table with S3Queue engine"""
        return cls(
            name=name,
            columns=columns,
            engine=S3QueueEngine(s3_path=s3_path, format=format, **kwargs),
            order_by=order_by
        )
    
    @classmethod
    def with_merge_tree(cls,
                        name: str,
                        columns: Dict[str, str],
                        order_by: Optional[str] = None,
                        deduplicate: bool = False,
                        **kwargs) -> 'TableConfig':
        """Create a table with MergeTree or ReplacingMergeTree engine"""
        engine = ReplacingMergeTreeEngine() if deduplicate else MergeTreeEngine()
        return cls(
            name=name,
            columns=columns,
            engine=engine,
            order_by=order_by
        )

# ==========================
# Legacy API Support (Deprecated)
# ==========================

@dataclass
class S3QueueEngineConfig:
    """Legacy S3Queue configuration (deprecated - use S3QueueEngine instead)"""
    path: str  # S3 path pattern (e.g., 's3://bucket/data/*.json')
    format: str  # Data format (e.g., 'JSONEachRow', 'CSV', etc.)
    # Optional S3 access credentials - can be NOSIGN for public buckets
    aws_access_key_id: Optional[str] = None
    aws_secret_access_key: Optional[str] = None
    # Optional compression
    compression: Optional[str] = None
    # Optional headers
    headers: Optional[Dict[str, str]] = None
    # Engine-specific settings
    settings: Optional[Dict[str, Any]] = None  # S3Queue engine settings

@dataclass
class TableCreateOptions:
    name: str
    columns: Dict[str, str]
    engine: Optional[ClickHouseEngines] = ClickHouseEngines.MergeTree
    order_by: Optional[str] = None
    s3_queue_engine_config: Optional[S3QueueEngineConfig] = None  # Required when engine is S3Queue

    def __post_init__(self):
        """Validate S3Queue configuration"""
        if self.engine == ClickHouseEngines.S3Queue and self.s3_queue_engine_config is None:
            raise ValueError(
                "s3_queue_engine_config is required when using ClickHouseEngines.S3Queue engine. "
                "Please provide s3_queue_engine_config with path, format, and optional settings."
            )

# ==========================
# Backward Compatibility Layer
# ==========================

def is_new_config(config: Any) -> bool:
    """Check if configuration uses new API"""
    if isinstance(config, TableConfig):
        return True
    if hasattr(config, 'engine') and isinstance(getattr(config, 'engine'), EngineConfig):
        return True
    return False

def migrate_legacy_config(legacy: TableCreateOptions) -> TableConfig:
    """Convert legacy configuration to new format"""
    
    # Show deprecation warning
    warnings.warn(
        "Using deprecated TableCreateOptions. Please migrate to TableConfig:\n"
        "- For S3Queue: Use TableConfig.with_s3_queue()\n"
        "- For deduplication: Use engine=ReplacingMergeTreeEngine()\n"
        "See documentation for examples.",
        DeprecationWarning,
        stacklevel=2
    )
    
    # Handle S3Queue with separate config
    if legacy.engine == ClickHouseEngines.S3Queue and legacy.s3_queue_engine_config:
        s3_config = legacy.s3_queue_engine_config
        return TableConfig(
            name=legacy.name,
            columns=legacy.columns,
            engine=S3QueueEngine(
                s3_path=s3_config.path,
                format=s3_config.format,
                aws_access_key_id=s3_config.aws_access_key_id,
                aws_secret_access_key=s3_config.aws_secret_access_key,
                compression=s3_config.compression,
                headers=s3_config.headers,
                s3_settings=s3_config.settings or {}
            ),
            order_by=legacy.order_by
        )
    
    # Map legacy engine enum to new engine classes
    engine_map = {
        ClickHouseEngines.MergeTree: MergeTreeEngine(),
        ClickHouseEngines.ReplacingMergeTree: ReplacingMergeTreeEngine(),
        ClickHouseEngines.AggregatingMergeTree: AggregatingMergeTreeEngine(),
        ClickHouseEngines.SummingMergeTree: SummingMergeTreeEngine(),
    }
    
    engine = engine_map.get(legacy.engine) if legacy.engine else MergeTreeEngine()
    if engine is None:
        engine = MergeTreeEngine()
    
    return TableConfig(
        name=legacy.name,
        columns=legacy.columns,
        engine=engine,
        order_by=legacy.order_by
    )

def normalize_config(config: Union[TableConfig, TableCreateOptions]) -> TableConfig:
    """Normalize any configuration format to new API"""
    if is_new_config(config):
        return config  # type: ignore
    return migrate_legacy_config(config)  # type: ignore

@dataclass
class AggregationCreateOptions:
    table_create_options: TableCreateOptions
    materialized_view_name: str
    select: str

@dataclass
class AggregationDropOptions:
    view_name: str
    table_name: str

@dataclass
class MaterializedViewCreateOptions:
    name: str
    destination_table: str
    select: str

@dataclass
class PopulateTableOptions:
    destination_table: str
    select: str

@dataclass
class Blocks:
    teardown: list[str]
    setup: list[str]

def drop_aggregation(options: AggregationDropOptions) -> list[str]:
    """
    Drops an aggregation's view & underlying table.
    """
    return [drop_view(options.view_name), drop_table(options.table_name)]

def drop_table(name: str) -> str:
    """
    Drops an existing table if it exists.
    """
    return f"DROP TABLE IF EXISTS {name}".strip()

def drop_view(name: str) -> str:
    """
    Drops an existing view if it exists.
    """
    return f"DROP VIEW IF EXISTS {name}".strip()

def create_aggregation(options: AggregationCreateOptions) -> list[str]:
    """
    Creates an aggregation which includes a table, materialized view, and initial data load.
    """
    return [
        create_table(options.table_create_options),
        create_materialized_view(MaterializedViewCreateOptions(
            name=options.materialized_view_name,
            destination_table=options.table_create_options.name,
            select=options.select
        )),
        populate_table(PopulateTableOptions(
            destination_table=options.table_create_options.name,
            select=options.select
        )),
    ]

def create_materialized_view(options: MaterializedViewCreateOptions) -> str:
    """
    Creates a materialized view.
    """
    return f"CREATE MATERIALIZED VIEW IF NOT EXISTS {options.name} \nTO {options.destination_table}\nAS {options.select}".strip()

def create_table(options: TableCreateOptions) -> str:
    """
    Creates a new table with default MergeTree engine.
    """
    column_definitions = ",\n".join([f"{name} {type}" for name, type in options.columns.items()])
    order_by_clause = f"ORDER BY {options.order_by}" if options.order_by else ""
    engine = options.engine.value if options.engine else "MergeTree"

    return f"""
    CREATE TABLE IF NOT EXISTS {options.name} 
    (
      {column_definitions}
    )
    ENGINE = {engine}()
    {order_by_clause}
    """.strip()

def populate_table(options: PopulateTableOptions) -> str:
    """
    Populates a table with data.
    """
    return f"INSERT INTO {options.destination_table}\n{options.select}".strip()