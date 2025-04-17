import dataclasses
from datetime import datetime
from enum import Enum
from .main import IngestionFormat
from typing import Any, Generic, Optional, TypeVar, Callable, Union, Tuple, Annotated
from pydantic import BaseModel
from pydantic.fields import FieldInfo
from pydantic.json_schema import JsonSchemaValue

from moose_lib import ClickHouseEngines

_tables: dict[str, "OlapTable"] = {}
_streams: dict[str, "Stream"] = {}
_ingest_apis: dict[str, "IngestApi"] = {}
_egress_apis: dict[str, "ConsumptionApi"] = {}
_sql_resources: dict[str, "SqlResource"] = {}

T = TypeVar('T', bound=BaseModel)
U = TypeVar('U', bound=BaseModel)
type ZeroOrMany[T] = Union[T, list[T], None]


class Columns(Generic[T]):
    """Provides runtime checked column name access for Moose resources."""
    _fields: dict[str, FieldInfo]

    def __init__(self, model: type[T]):
        self._fields = model.model_fields

    def __getattr__(self, item: str) -> str:
        if item in self._fields:
            return item  # or some Column representation
        raise AttributeError(f"{item} is not a valid column name")


class BaseTypedResource(Generic[T]):
    """Base class for Moose resources that are typed with a Pydantic model."""
    _t: type[T]
    name: str

    @classmethod
    def _get_type(cls, keyword_args: dict):
        t = keyword_args.get('t')
        if t is None:
            raise ValueError(f"Use `{cls.__name__}[T](name='...')` to supply the Pydantic model type`")
        if not isinstance(t, type) or not issubclass(t, BaseModel):
            raise ValueError(f"{t} is not a Pydantic model")
        return t

    @property
    def model_type(self) -> type[T]:
        """Get the Pydantic model type associated with this resource."""
        return self._t

    def _set_type(self, name: str, t: type[T]):
        self._t = t
        self.name = name

    def __class_getitem__(cls, item: type[BaseModel]):
        def curried_constructor(*args, **kwargs):
            return cls(t=item, *args, **kwargs)

        return curried_constructor


class TypedMooseResource(BaseTypedResource, Generic[T]):
    """Base class for Moose resources that have columns."""
    columns: Columns[T]

    def _set_type(self, name: str, t: type[T]):
        super()._set_type(name, t)
        self.columns = Columns[T](self._t)


class OlapConfig(BaseModel):
    """Configuration for OLAP tables."""
    order_by_fields: list[str] = []
    # equivalent to setting `engine=ClickHouseEngines.ReplacingMergeTree`
    deduplicate: bool = False
    engine: Optional[ClickHouseEngines] = None
    version: Optional[str] = None


class OlapTable(TypedMooseResource, Generic[T]):
    config: OlapConfig

    def __init__(self, name: str, config: OlapConfig = OlapConfig(), **kwargs):
        super().__init__()
        self._set_type(name, self._get_type(kwargs))
        self.config = config
        _tables[name] = self


class StreamConfig(BaseModel):
    """Configuration for data streams."""
    parallelism: int = 1
    retention_period: int = 60 * 60 * 24 * 7  # 7 days
    destination: Optional[OlapTable[Any]] = None
    version: Optional[str] = None

class TransformConfig(BaseModel):
    """Configuration for transformations."""
    version: Optional[str] = None

class ConsumerConfig(BaseModel):
    """Configuration for consumers."""
    version: Optional[str] = None

@dataclasses.dataclass
class _RoutedMessage:
    """Internal class representing a message routed to a specific stream."""
    destination: "Stream[Any]"
    values: ZeroOrMany[Any]


@dataclasses.dataclass
class ConsumerEntry(Generic[T]):
    """Internal class representing a consumer with its configuration."""
    consumer: Callable[[T], None]
    config: ConsumerConfig


@dataclasses.dataclass
class TransformEntry(Generic[T]):
    """Internal class representing a transformation with its configuration."""
    destination: "Stream[Any]"
    transformation: Callable[[T], ZeroOrMany[Any]]
    config: TransformConfig


class Stream(TypedMooseResource, Generic[T]):
    """
    A data stream that can transform and route messages.
    
    Supports:
    - Single transformations to other streams
    - Multi-route transformations where one record can be sent to multiple destinations
    - Optional connection to a destination table
    """
    config: StreamConfig
    transformations: dict[str, list[TransformEntry[T]]]
    consumers: list[ConsumerEntry[T]]
    _multipleTransformations: Optional[Callable[[T], list[_RoutedMessage]]] = None

    def __init__(self, name: str, config: StreamConfig = StreamConfig(), **kwargs):
        super().__init__()
        self._set_type(name, self._get_type(kwargs))
        self.config = config
        self.consumers = []
        self.transformations = {}
        _streams[name] = self

    def add_transform(self, destination: "Stream[U]", transformation: Callable[[T], ZeroOrMany[U]], config: TransformConfig = TransformConfig()):
        """Add a transformation that sends records to a single destination stream."""
        if destination.name in self.transformations:
            existing_transforms = self.transformations[destination.name]
            # Check if a transform with this version already exists
            has_version = any(t.config.version == config.version for t in existing_transforms)
            if not has_version:
                existing_transforms.append(TransformEntry(destination=destination, transformation=transformation, config=config))
        else:
            self.transformations[destination.name] = [TransformEntry(destination=destination, transformation=transformation, config=config)]

    def add_consumer(self, consumer: Callable[[T], None], config: ConsumerConfig = ConsumerConfig()):
        """Add a consumer that will be called for each record in the stream."""
        has_version = any(c.config.version == config.version for c in self.consumers)
        if not has_version:
            self.consumers.append(ConsumerEntry(consumer=consumer, config=config))

    def has_consumers(self) -> bool:
        """Check if the stream has any consumers."""
        return len(self.consumers) > 0

    def routed(self, values: ZeroOrMany[T]) -> _RoutedMessage:
        """Create a routed message targeting this stream.
        
        This method is used to create the return values for multi_transform functions.
        """
        return _RoutedMessage(destination=self, values=values)

    def set_multi_transform(self, transformation: Callable[[T], list[_RoutedMessage]]):
        """Set a transformation that can route records to multiple destinations.
        
        The transformation function must return a list of _RoutedMessage objects.
        Use the routed() method to create these objects, for example:
        
            def multi_transform(record):
                return [
                    stream1.routed(transform1(record)),
                    stream2.routed(transform2(record))
                ]
            stream.set_multi_transform(multi_transform)
        """
        self._multipleTransformations = transformation


class IngestConfig(BaseModel):
    """Basic ingestion configuration."""
    format: IngestionFormat = IngestionFormat.JSON
    version: Optional[str] = None


@dataclasses.dataclass
class IngestConfigWithDestination[T: BaseModel]:
    """Ingestion configuration that includes a destination stream."""
    destination: Stream[T]
    format: IngestionFormat = IngestionFormat.JSON
    version: Optional[str] = None


class IngestPipelineConfig(BaseModel):
    """Configuration for creating a complete data pipeline with table, stream and ingestion."""
    table: bool | OlapConfig = True
    stream: bool | StreamConfig = True
    ingest: bool | IngestConfig = True
    version: Optional[str] = None


class IngestApi(TypedMooseResource, Generic[T]):
    """Configures an ingestion endpoint that writes to a stream."""
    config: IngestConfigWithDestination[T]

    def __init__(self, name: str, config: IngestConfigWithDestination[T], **kwargs):
        super().__init__()
        self._set_type(name, self._get_type(kwargs))
        self.config = config
        _ingest_apis[name] = self


class IngestPipeline(TypedMooseResource, Generic[T]):
    """
    Creates a complete data pipeline with:
    - An optional OLAP table for storage
    - An optional stream for processing
    - An optional ingestion API
    
    The components are automatically connected when created.
    """
    table: Optional[OlapTable[T]] = None
    stream: Optional[Stream[T]] = None
    ingest_api: Optional[IngestApi[T]] = None

    def get_table(self) -> OlapTable[T]:
        """Get the OLAP table, raising an error if it was not configured."""
        if self.table is None:
            raise ValueError("Table was not configured for this pipeline")
        return self.table

    def get_stream(self) -> Stream[T]:
        """Get the stream, raising an error if it was not configured."""
        if self.stream is None:
            raise ValueError("Stream was not configured for this pipeline")
        return self.stream

    def get_ingest_api(self) -> IngestApi[T]:
        """Get the ingestion API, raising an error if it was not configured."""
        if self.ingest_api is None:
            raise ValueError("Ingest API was not configured for this pipeline")
        return self.ingest_api

    def __init__(self, name: str, config: IngestPipelineConfig, **kwargs):
        super().__init__()
        self._set_type(name, self._get_type(kwargs))
        if config.table:
            table_config = OlapConfig() if config.table is True else config.table
            if config.version:
                table_config.version = config.version
            self.table = OlapTable(name, table_config, t=self._t)
        if config.stream:
            stream_config = StreamConfig() if config.stream is True else config.stream
            if config.table and stream_config.destination is not None:
                raise ValueError("The destination of the stream should be the table created in the IngestPipeline")
            stream_config.destination = self.table
            if config.version:
                stream_config.version = config.version
            self.stream = Stream(name, stream_config, t=self._t)
        if config.ingest:
            if self.stream is None:
                raise ValueError("Ingest API needs a stream to write to.")
            ingest_config_dict = (
                IngestConfig() if config.ingest is True else config.ingest
            ).model_dump()
            ingest_config_dict["destination"] = self.stream
            if config.version:
                ingest_config_dict["version"] = config.version
            ingest_config = IngestConfigWithDestination(**ingest_config_dict)
            self.ingest_api = IngestApi(name, ingest_config, t=self._t)


class EgressConfig(BaseModel):
    """Configuration for Consumption APIs."""
    version: Optional[str] = None


class ConsumptionApi(BaseTypedResource, Generic[T, U]):
    """Configures a Consumption API that can be used to query the data."""
    config: EgressConfig
    query_function: Callable[..., U]
    _u: type[U]

    def __class_getitem__(cls, items):
        # Handle two type parameters
        if not isinstance(items, tuple) or len(items) != 2:
            raise ValueError(f"Use `{cls.__name__}[T, U](name='...')` to supply both input and output types")
        input_type, output_type = items

        def curried_constructor(*args, **kwargs):
            return cls(t=(input_type, output_type), *args, **kwargs)

        return curried_constructor

    def __init__(
            self,
            name: str,
            query_function: Callable[..., U],
            config: EgressConfig = EgressConfig(),
            **kwargs
    ):
        super().__init__()
        self._set_type(name, self._get_type(kwargs))
        self.config = config
        self.query_function = query_function
        _egress_apis[name] = self

    @classmethod
    def _get_type(cls, keyword_args: dict):
        t = keyword_args.get('t')
        if not isinstance(t, tuple) or len(t) != 2:
            raise ValueError(f"Use `{cls.__name__}[T, U](name='...')` to supply both input and output types")

        input_type, output_type = t
        if not isinstance(input_type, type) or not issubclass(input_type, BaseModel):
            raise ValueError(f"Input type {input_type} is not a Pydantic model")
        if not isinstance(output_type, type) or not issubclass(output_type, BaseModel):
            raise ValueError(f"Output type {output_type} is not a Pydantic model")
        return t

    def _set_type(self, name: str, t: tuple[type[T], type[U]]):
        input_type, output_type = t
        self._t = input_type
        self._u = output_type
        self.name = name

    @property
    def return_type(self) -> type[U]:
        """Get the return type associated with this resource."""
        return self._u

    def get_response_schema(self) -> JsonSchemaValue:
        from pydantic.type_adapter import TypeAdapter
        return TypeAdapter(self.return_type).json_schema(
            ref_template='#/components/schemas/{model}'
        )


def _get_consumption_api(name: str) -> Optional[ConsumptionApi]:
    return _egress_apis.get(name)


class SqlResource:
    """Base class for SQL resources like views and tables."""
    setup: list[str]
    teardown: list[str]
    name: str

    def __init__(self, name: str, setup: list[str], teardown: list[str]):
        self.name = name
        self.setup = setup
        self.teardown = teardown
        _sql_resources[name] = self


class View(SqlResource):
    """A materialized view in the database."""

    def __init__(self, name: str, select_statement: str):
        setup = [
            f"CREATE MATERIALIZED VIEW IF NOT EXISTS {name} AS {select_statement}".strip()
        ]
        teardown = [f"DROP VIEW IF EXISTS {name}"]
        super().__init__(name, setup, teardown)


class MaterializedViewOptions(BaseModel):
    """Configuration options for materialized views."""
    select_statement: str
    table_name: str
    materialized_view_name: str
    engine: Optional[ClickHouseEngines] = None
    order_by_fields: Optional[list[str]] = None


class MaterializedView(SqlResource, BaseTypedResource, Generic[T]):
    """A materialized view with a typed target table."""
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

        super().__init__(options.materialized_view_name, setup, teardown)

        self.target_table = OlapTable(
            name=options.table_name,
            config=OlapConfig(
                order_by_fields=options.order_by_fields or [],
                engine=options.engine
            ),
            t=self._t
        )
