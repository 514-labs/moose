import dataclasses
from datetime import datetime
from enum import Enum
from typing import Any, Generic, Optional, TypeVar, Callable, Union, Tuple
from pydantic import BaseModel
from pydantic.fields import FieldInfo
from pydantic.json_schema import JsonSchemaValue

_tables: dict[str, "OlapTable"] = {}
_streams: dict[str, "Stream"] = {}
_ingest_apis: dict[str, "IngestApi"] = {}
_egress_apis: dict[str, "ConsumptionApi"] = {}

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


class IngestionFormat(Enum):
    """Supported formats for data ingestion."""
    JSON = "JSON"
    JSON_ARRAY = "JSON_ARRAY"


class OlapConfig(BaseModel):
    """Configuration for OLAP tables."""
    order_by_fields: list[str] = []
    deduplicate: bool = False


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


@dataclasses.dataclass
class _RoutedMessage:
    """Internal class representing a message routed to a specific stream."""
    destination: "Stream[Any]"
    values: ZeroOrMany[Any]


class Stream(TypedMooseResource, Generic[T]):
    """
    A data stream that can transform and route messages.
    
    Supports:
    - Single transformations to other streams
    - Multi-route transformations where one record can be sent to multiple destinations
    - Optional connection to a destination table
    """
    config: StreamConfig
    transformations: dict[str, Tuple["Stream[Any]", Callable[[T], ZeroOrMany[Any]]]] = {}
    _multipleTransformations: Optional[Callable[[T], list[_RoutedMessage]]] = None

    def __init__(self, name: str, config: StreamConfig = StreamConfig(), **kwargs):
        super().__init__()
        self._set_type(name, self._get_type(kwargs))
        self.config = config
        _streams[name] = self

    def add_transform(self, destination: "Stream[U]", transformation: Callable[[T], ZeroOrMany[U]]):
        """Add a transformation that sends records to a single destination stream."""
        self.transformations[destination.name] = (destination, transformation)

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


@dataclasses.dataclass
class IngestConfigWithDestination[T: BaseModel]:
    """Ingestion configuration that includes a destination stream."""
    destination: Stream[T]
    format: IngestionFormat = IngestionFormat.JSON


class DataModelConfigV2(BaseModel):
    """Configuration for creating a complete data pipeline with table, stream and ingestion."""
    table: bool | OlapConfig = True
    stream: bool | StreamConfig = True
    ingest: bool | IngestConfig = True


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

    def __init__(self, name: str, config: DataModelConfigV2, **kwargs):
        super().__init__()
        self._set_type(name, self._get_type(kwargs))
        if config.table:
            table_config = OlapConfig() if config.table is True else config.table
            self.table = OlapTable(name, table_config, t=self._t)
        if config.stream:
            stream_config = StreamConfig() if config.stream is True else config.stream
            if config.table and stream_config.destination is not None:
                raise ValueError("The destination of the stream should be the table created in the IngestPipeline")
            stream_config.destination = self.table
            self.stream = Stream(name, stream_config, t=self._t)
        if config.ingest:
            if self.stream is None:
                raise ValueError("Ingest API needs a stream to write to.")
            ingest_config_dict = (
                IngestConfig() if config.ingest is True else config.ingest
            ).model_dump()
            ingest_config_dict["destination"] = self.stream
            ingest_config = IngestConfigWithDestination(**ingest_config_dict)
            self.ingest_api = IngestApi(name, ingest_config, t=self._t)

class EgressConfig(BaseModel):
    """Configuration for Consumption APIs."""
    pass

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


def get_consumption_api(name: str) -> Optional[ConsumptionApi]:
    return _egress_apis.get(name)
