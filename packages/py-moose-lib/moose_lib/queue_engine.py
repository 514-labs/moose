"""
Queue Engine Types for Moose

Provides backend-agnostic queue table engine support for streaming data ingestion.
Currently supports S3Queue and AzureQueue with comprehensive configuration options.
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Dict, Optional, Union, List
from abc import ABC, abstractmethod


class ProcessingMode(Enum):
    ORDERED = "ordered"
    UNORDERED = "unordered"


class AfterProcessing(Enum):
    KEEP = "keep"
    DELETE = "delete"


@dataclass
class S3Credentials:
    """S3 authentication credentials"""
    role_arn: Optional[str] = None
    access_key_id: Optional[str] = None
    secret_access_key: Optional[str] = None
    session_token: Optional[str] = None


@dataclass
class QueueSource(ABC):
    """Base class for queue sources"""
    format: str
    extra_settings: Dict[str, str] = field(default_factory=dict)
    
    @abstractmethod
    def source_type(self) -> str:
        """Return the source type identifier"""
        pass


@dataclass
class S3QueueSource(QueueSource):
    """S3 queue source configuration"""
    path: str
    credentials: Optional[S3Credentials] = None
    
    def source_type(self) -> str:
        return "s3"


@dataclass
class AzureQueueSource(QueueSource):
    """Azure queue source configuration"""
    container: str
    path: str
    
    def source_type(self) -> str:
        return "azure"


@dataclass
class ProcessingConfig:
    """Processing behavior configuration"""
    mode: ProcessingMode = ProcessingMode.UNORDERED
    after_processing: AfterProcessing = AfterProcessing.KEEP
    retries: int = 0
    threads: Optional[int] = None
    parallel_inserts: bool = False
    buckets: Optional[int] = None


@dataclass
class CoordinationConfig:
    """Coordination and state management configuration"""
    path: Optional[str] = None
    tracked_files_limit: Optional[int] = 1000
    tracked_file_ttl_sec: Optional[int] = None
    cleanup_interval_min_ms: Optional[int] = 10000
    cleanup_interval_max_ms: Optional[int] = 30000


@dataclass
class MonitoringConfig:
    """Monitoring and polling configuration"""
    enable_logging: bool = False
    polling_min_timeout_ms: Optional[int] = 1000
    polling_max_timeout_ms: Optional[int] = 10000
    polling_backoff_ms: Optional[int] = 0


@dataclass
class QueueEngine:
    """Generic queue engine configuration"""
    source: QueueSource
    processing: ProcessingConfig = field(default_factory=ProcessingConfig)
    coordination: CoordinationConfig = field(default_factory=CoordinationConfig)
    monitoring: MonitoringConfig = field(default_factory=MonitoringConfig)

    def validate(self) -> List[str]:
        """Validate the queue engine configuration"""
        errors: List[str] = []
        
        # Validate source
        if isinstance(self.source, S3QueueSource):
            if not self.source.path:
                errors.append("S3 path is required")
            elif not self.source.path.startswith("s3://"):
                errors.append("S3 path must start with 's3://'")
        elif isinstance(self.source, AzureQueueSource):
            if not self.source.container:
                errors.append("Azure container is required")
            if not self.source.path:
                errors.append("Azure path is required")
        
        if not self.source.format:
            errors.append("Format is required")
        
        # Validate processing config
        if self.processing.buckets is not None and self.processing.buckets <= 0:
            errors.append("Buckets must be greater than 0")
        
        if self.processing.retries < 0:
            errors.append("Retries must be non-negative")
        
        if self.processing.threads is not None and self.processing.threads <= 0:
            errors.append("Threads must be greater than 0")
        
        # Validate ordered mode constraints
        if (self.processing.mode == ProcessingMode.ORDERED and 
            self.processing.buckets is not None and 
            self.processing.threads is not None):
            if self.processing.buckets < self.processing.threads:
                errors.append("For ordered mode, buckets should be >= threads")
        
        return errors

    def source_type(self) -> str:
        """Get the source type as a string"""
        return self.source.source_type()


def create_s3_queue_engine(
    path: str,
    format: str,
    credentials: Optional[S3Credentials] = None,
    processing: Optional[ProcessingConfig] = None,
    coordination: Optional[CoordinationConfig] = None,
    monitoring: Optional[MonitoringConfig] = None,
    extra_settings: Optional[Dict[str, str]] = None
) -> QueueEngine:
    """Create an S3Queue engine configuration with sensible defaults"""
    return QueueEngine(
        source=S3QueueSource(
            path=path,
            format=format,
            credentials=credentials,
            extra_settings=extra_settings or {}
        ),
        processing=processing or ProcessingConfig(),
        coordination=coordination or CoordinationConfig(),
        monitoring=monitoring or MonitoringConfig()
    )


def create_azure_queue_engine(
    container: str,
    path: str,
    format: str,
    processing: Optional[ProcessingConfig] = None,
    coordination: Optional[CoordinationConfig] = None,
    monitoring: Optional[MonitoringConfig] = None,
    extra_settings: Optional[Dict[str, str]] = None
) -> QueueEngine:
    """Create an AzureQueue engine configuration with sensible defaults"""
    return QueueEngine(
        source=AzureQueueSource(
            container=container,
            path=path,
            format=format,
            extra_settings=extra_settings or {}
        ),
        processing=processing or ProcessingConfig(),
        coordination=coordination or CoordinationConfig(),
        monitoring=monitoring or MonitoringConfig()
    )


# Common ClickHouse formats supported by queue engines
SUPPORTED_FORMATS = [
    "JSONEachRow",
    "CSV", 
    "TSV",
    "Parquet",
    "JSONStringsEachRow",
    "JSONCompactEachRow",
    "CSVWithNames",
    "TSVWithNames",
    "TabSeparated",
    "TabSeparatedWithNames",
    "TSKV",
    "Avro",
    "ORC",
]


class ExampleConfigs:
    """Example usage configurations for common scenarios"""
    
    @staticmethod
    def s3_json_logs(bucket: str, prefix: str) -> QueueEngine:
        """Basic S3Queue for JSON logs"""
        return create_s3_queue_engine(f"s3://{bucket}/{prefix}/*.json", "JSONEachRow")
    
    @staticmethod
    def s3_high_throughput(bucket: str, prefix: str) -> QueueEngine:
        """High-throughput S3Queue with parallel processing"""
        return create_s3_queue_engine(
            f"s3://{bucket}/{prefix}/*.json",
            "JSONEachRow",
            processing=ProcessingConfig(
                mode=ProcessingMode.UNORDERED,
                threads=8,
                parallel_inserts=True,
                buckets=16,
                retries=3
            ),
            monitoring=MonitoringConfig(
                enable_logging=True,
                polling_min_timeout_ms=500,
                polling_max_timeout_ms=5000
            )
        )
    
    @staticmethod
    def s3_ordered_processing(bucket: str, prefix: str) -> QueueEngine:
        """Ordered processing for sequential data"""
        return create_s3_queue_engine(
            f"s3://{bucket}/{prefix}/*.csv",
            "CSV",
            processing=ProcessingConfig(
                mode=ProcessingMode.ORDERED,
                buckets=4,
                threads=2,
                retries=5
            ),
            coordination=CoordinationConfig(
                tracked_file_ttl_sec=3600,  # 1 hour
                tracked_files_limit=5000
            )
        )
    
    @staticmethod
    def s3_development(bucket: str, prefix: str) -> QueueEngine:
        """Development/testing configuration"""
        return create_s3_queue_engine(
            f"s3://{bucket}/{prefix}/*.json",
            "JSONEachRow",
            processing=ProcessingConfig(retries=1),
            monitoring=MonitoringConfig(
                enable_logging=True,
                polling_min_timeout_ms=2000,
                polling_max_timeout_ms=10000
            )
        )