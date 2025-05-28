"""
from __future__ import annotations

Moose Data Model v2 (dmv2) - Python Implementation

This module provides the Python classes for defining Moose v2 data model resources,
including OLAP tables, streams, ingestion/consumption APIs, pipelines, and SQL views.
It mirrors the functionality of the TypeScript `dmv2` module, enabling the definition
of data infrastructure using Python and Pydantic models.
"""
import dataclasses
import datetime
from typing import Any, Generic, Optional, TypeVar, Callable, Union, Literal
from pydantic import BaseModel, ConfigDict, AliasGenerator
from pydantic.alias_generators import to_camel
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
    """Provides runtime checked column name access for Moose resources.

    Instead of using string literals for column names, you can use attribute access
    on this object, which will verify the name against the Pydantic model's fields.

    Example:
        >>> class MyModel(BaseModel):
        ...     user_id: int
        ...     event_name: str
        >>> cols = Columns(MyModel)
        >>> print(cols.user_id)  # Output: user_id
        >>> print(cols.non_existent) # Raises AttributeError

    Args:
        model: The Pydantic model type whose fields represent the columns.
    """
    _fields: dict[str, FieldInfo]

    def __init__(self, model: type[T]):
        self._fields = model.model_fields

    def __getattr__(self, item: str) -> str:
        if item in self._fields:
            return item  # or some Column representation
        raise AttributeError(f"{item} is not a valid column name")


class BaseTypedResource(Generic[T]):
    """Base class for Moose resources that are typed with a Pydantic model.

    Handles the association of a Pydantic model `T` with a Moose resource,
    providing type validation and access to the model type.

    Attributes:
        name (str): The name of the Moose resource.
    """
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
        """Internal method to set the resource name and associated Pydantic type."""
        self._t = t
        self.name = name

    def __class_getitem__(cls, item: type[BaseModel]):
        def curried_constructor(*args, **kwargs):
            return cls(t=item, *args, **kwargs)

        return curried_constructor


class TypedMooseResource(BaseTypedResource, Generic[T]):
    """Base class for Moose resources that have columns derived from a Pydantic model.

    Extends `BaseTypedResource` by adding a `Columns` helper for type-safe
    column name access.

    Attributes:
        columns (Columns[T]): An object providing attribute access to column names.
    """
    columns: Columns[T]

    def _set_type(self, name: str, t: type[T]):
        super()._set_type(name, t)
        self.columns = Columns[T](self._t)


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


class StreamConfig(BaseModel):
    """Configuration for data streams (e.g., Redpanda topics).

    Attributes:
        parallelism: Number of partitions for the stream.
        retention_period: Data retention period in seconds (default: 7 days).
        destination: Optional `OlapTable` where stream messages should be automatically ingested.
        version: Optional version string for tracking configuration changes.
        metadata: Optional metadata for the stream.
    """
    parallelism: int = 1
    retention_period: int = 60 * 60 * 24 * 7  # 7 days
    destination: Optional[OlapTable[Any]] = None
    version: Optional[str] = None
    metadata: Optional[dict] = None


class TransformConfig(BaseModel):
    """Configuration for stream transformations.

    Attributes:
        version: Optional version string to identify a specific transformation.
                 Allows multiple transformations to the same destination if versions differ.
    """
    version: Optional[str] = None
    dead_letter_queue: "Optional[DeadLetterQueue]" = None
    model_config = ConfigDict(arbitrary_types_allowed=True)
    metadata: Optional[dict] = None


class ConsumerConfig(BaseModel):
    """Configuration for stream consumers.

    Attributes:
        version: Optional version string to identify a specific consumer.
                 Allows multiple consumers if versions differ.
    """
    version: Optional[str] = None
    dead_letter_queue: "Optional[DeadLetterQueue]" = None
    model_config = ConfigDict(arbitrary_types_allowed=True)


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
    """Represents a data stream (e.g., a Redpanda topic) typed with a Pydantic model.

    Allows defining transformations to other streams and adding consumers.

    Args:
        name: The name of the stream.
        config: Configuration options for the stream (parallelism, retention, destination).
        t: The Pydantic model defining the stream message schema (passed via `Stream[MyModel](...)`).

    Attributes:
        config (StreamConfig): Configuration settings for this stream.
        transformations (dict[str, list[TransformEntry[T]]]): Dictionary mapping destination stream names
                                                            to lists of transformation functions.
        consumers (list[ConsumerEntry[T]]): List of consumers attached to this stream.
        columns (Columns[T]): Helper for accessing message field names safely.
        name (str): The name of the stream.
        model_type (type[T]): The Pydantic model associated with this stream.
    """
    config: StreamConfig
    transformations: dict[str, list[TransformEntry[T]]]
    consumers: list[ConsumerEntry[T]]
    _multipleTransformations: Optional[Callable[[T], list[_RoutedMessage]]] = None

    def __init__(self, name: str, config: StreamConfig = StreamConfig(), **kwargs):
        super().__init__()
        self._set_type(name, self._get_type(kwargs))
        self.config = config
        self.metadata = config.metadata
        self.consumers = []
        self.transformations = {}
        _streams[name] = self

    def add_transform(self, destination: "Stream[U]", transformation: Callable[[T], ZeroOrMany[U]],
                      config: TransformConfig = None):
        """Adds a transformation step from this stream to a destination stream.

        The transformation function receives a record of type `T` and should return
        a record of type `U`, a list of `U` records, or `None` to filter.

        Args:
            destination: The target `Stream` for the transformed records.
            transformation: A callable that performs the transformation.
            config: Optional configuration, primarily for setting a version.
        """
        config = config or TransformConfig()
        if destination.name in self.transformations:
            existing_transforms = self.transformations[destination.name]
            # Check if a transform with this version already exists
            has_version = any(t.config.version == config.version for t in existing_transforms)
            if not has_version:
                existing_transforms.append(
                    TransformEntry(destination=destination, transformation=transformation, config=config))
        else:
            self.transformations[destination.name] = [
                TransformEntry(destination=destination, transformation=transformation, config=config)]

    def add_consumer(self, consumer: Callable[[T], None], config: ConsumerConfig = None):
        """Adds a consumer function to be executed for each record in the stream.

        Consumers are typically used for side effects like logging or triggering external actions.

        Args:
            consumer: A callable that accepts a record of type `T`.
            config: Optional configuration, primarily for setting a version.
        """
        config = config or ConsumerConfig()
        has_version = any(c.config.version == config.version for c in self.consumers)
        if not has_version:
            self.consumers.append(ConsumerEntry(consumer=consumer, config=config))

    def has_consumers(self) -> bool:
        """Checks if any consumers have been added to this stream.

        Returns:
            True if the stream has one or more consumers, False otherwise.
        """
        return len(self.consumers) > 0

    def routed(self, values: ZeroOrMany[T]) -> _RoutedMessage:
        """Creates a `_RoutedMessage` for use in multi-transform functions.

        Wraps the value(s) to be sent with this stream as the destination.

        Args:
            values: A single record, a list of records, or None.

        Returns:
            A `_RoutedMessage` object.
        """
        return _RoutedMessage(destination=self, values=values)

    def set_multi_transform(self, transformation: Callable[[T], list[_RoutedMessage]]):
        """Sets a transformation function capable of routing records to multiple streams.

        The provided function takes a single input record (`T`) and must return a list
        of `_RoutedMessage` objects, created using the `.routed()` method of the
        target streams.

        Example:
            def my_multi_transform(record: InputModel) -> list[_RoutedMessage]:
                output1 = transform_for_stream1(record)
                output2 = transform_for_stream2(record)
                return [
                    stream1.routed(output1),
                    stream2.routed(output2)
                ]
            input_stream.set_multi_transform(my_multi_transform)

        Note: Only one multi-transform function can be set per stream.

        Args:
            transformation: The multi-routing transformation function.
        """
        self._multipleTransformations = transformation


class DeadLetterModel(BaseModel, Generic[T]):
    model_config = ConfigDict(alias_generator=AliasGenerator(
        serialization_alias=to_camel,
    ))
    original_record: Any
    error_message: str
    error_type: str
    failed_at: datetime.datetime
    source: Literal["api", "transform", "table"]

    def as_typed(self) -> T:
        return self._t.model_validate(self.original_record)


class DeadLetterQueue(Stream, Generic[T]):
    """A specialized Stream for handling failed records.
    
    Dead letter queues store records that failed during processing, along with
    error information to help diagnose and potentially recover from failures.
    
    Attributes:
        All attributes inherited from Stream.
    """

    _model_type: type[T]

    def __init__(self, name: str, config: StreamConfig = StreamConfig(), **kwargs):
        """Initialize a new DeadLetterQueue.
        
        Args:
            name: The name of the dead letter queue stream.
            config: Configuration for the stream.
        """
        self._model_type = self._get_type(kwargs)
        kwargs["t"] = DeadLetterModel[self._model_type]
        super().__init__(name, config, **kwargs)

    def add_transform(self, destination: Stream[U], transformation: Callable[[DeadLetterModel[T]], ZeroOrMany[U]],
                      config: TransformConfig = None):
        def wrapped_transform(record: DeadLetterModel[T]):
            record._t = self._model_type
            return transformation(record)

        config = config or TransformConfig()
        super().add_transform(destination, wrapped_transform, config)

    def add_consumer(self, consumer: Callable[[DeadLetterModel[T]], None], config: ConsumerConfig = None):
        def wrapped_consumer(record: DeadLetterModel[T]):
            record._t = self._model_type
            return consumer(record)

        config = config or ConsumerConfig()
        super().add_consumer(wrapped_consumer, config)

    def set_multi_transform(self, transformation: Callable[[DeadLetterModel[T]], list[_RoutedMessage]]):
        def wrapped_transform(record: DeadLetterModel[T]):
            record._t = self._model_type
            return transformation(record)

        super().set_multi_transform(wrapped_transform)


class IngestConfig(BaseModel):
    """Basic configuration for an ingestion point.

    Attributes:
        version: Optional version string.
        metadata: Optional metadata for the ingestion point.
    """
    version: Optional[str] = None
    metadata: Optional[dict] = None


@dataclasses.dataclass
class IngestConfigWithDestination[T: BaseModel]:
    """Ingestion configuration that includes the mandatory destination stream.

    Attributes:
        destination: The `Stream` where ingested data will be sent.
        version: Optional version string.
        metadata: Optional metadata for the ingestion configuration.
    """
    destination: Stream[T]
    version: Optional[str] = None
    metadata: Optional[dict] = None


class IngestPipelineConfig(BaseModel):
    """Configuration for creating a complete ingestion pipeline.

    Defines which components (table, stream, ingest API) should be created.
    Set a component to `True` for default settings, `False` to disable, or provide
    a specific config object (`OlapConfig`, `StreamConfig`, `IngestConfig`).

    Attributes:
        table: Configuration for the OLAP table component.
        stream: Configuration for the stream component.
        ingest: Configuration for the ingest API component.
        version: Optional version string applied to all created components.
        metadata: Optional metadata for the ingestion pipeline.
    """
    table: bool | OlapConfig = True
    stream: bool | StreamConfig = True
    ingest: bool | IngestConfig = True
    version: Optional[str] = None
    metadata: Optional[dict] = None


class IngestApi(TypedMooseResource, Generic[T]):
    """Represents an Ingestion API endpoint typed with a Pydantic model.

    This endpoint receives data (matching schema `T`) and sends it to a configured
    destination stream.

    Args:
        name: The name of the ingestion API endpoint.
        config: Configuration specifying the destination stream and data format.
        t: The Pydantic model defining the expected input data schema
           (passed via `IngestApi[MyModel](...)`).

    Attributes:
        config (IngestConfigWithDestination[T]): The configuration for this API.
        columns (Columns[T]): Helper for accessing input field names safely.
        name (str): The name of the API.
        model_type (type[T]): The Pydantic model associated with this API's input.
    """
    config: IngestConfigWithDestination[T]

    def __init__(self, name: str, config: IngestConfigWithDestination[T], **kwargs):
        super().__init__()
        self._set_type(name, self._get_type(kwargs))
        self.config = config
        self.metadata = getattr(config, 'metadata', None)
        _ingest_apis[name] = self


class IngestPipeline(TypedMooseResource, Generic[T]):
    """Creates and configures a linked Table, Stream, and Ingest API pipeline.

    Simplifies the common pattern of ingesting data through an API, processing it
    in a stream, and storing it in a table.

    Args:
        name: The base name used for all created components (table, stream, API).
        config: Specifies which components to create and their configurations.
        t: The Pydantic model defining the data schema for all components
           (passed via `IngestPipeline[MyModel](...)`).

    Attributes:
        table: The created `OlapTable` instance, if configured.
        stream: The created `Stream` instance, if configured.
        ingest_api: The created `IngestApi` instance, if configured.
        columns (Columns[T]): Helper for accessing data field names safely.
        name (str): The base name of the pipeline.
        model_type (type[T]): The Pydantic model associated with this pipeline.
    """
    table: Optional[OlapTable[T]] = None
    stream: Optional[Stream[T]] = None
    ingest_api: Optional[IngestApi[T]] = None
    metadata: Optional[dict] = None

    def get_table(self) -> OlapTable[T]:
        """Retrieves the pipeline's OLAP table component.

        Raises:
            ValueError: If the table was not configured for this pipeline.

        Returns:
            The `OlapTable` instance.
        """
        if self.table is None:
            raise ValueError("Table was not configured for this pipeline")
        return self.table

    def get_stream(self) -> Stream[T]:
        """Retrieves the pipeline's stream component.

        Raises:
            ValueError: If the stream was not configured for this pipeline.

        Returns:
            The `Stream` instance.
        """
        if self.stream is None:
            raise ValueError("Stream was not configured for this pipeline")
        return self.stream

    def get_ingest_api(self) -> IngestApi[T]:
        """Retrieves the pipeline's Ingestion API component.

        Raises:
            ValueError: If the Ingest API was not configured for this pipeline.

        Returns:
            The `IngestApi` instance.
        """
        if self.ingest_api is None:
            raise ValueError("Ingest API was not configured for this pipeline")
        return self.ingest_api

    def __init__(self, name: str, config: IngestPipelineConfig, **kwargs):
        super().__init__()
        self._set_type(name, self._get_type(kwargs))
        self.metadata = config.metadata
        table_metadata = config.metadata
        stream_metadata = config.metadata
        ingest_metadata = config.metadata
        if config.table:
            table_config = OlapConfig() if config.table is True else config.table
            if config.version:
                table_config.version = config.version
            table_config.metadata = table_metadata
            self.table = OlapTable(name, table_config, t=self._t)
        if config.stream:
            stream_config = StreamConfig() if config.stream is True else config.stream
            if config.table and stream_config.destination is not None:
                raise ValueError("The destination of the stream should be the table created in the IngestPipeline")
            stream_config.destination = self.table
            if config.version:
                stream_config.version = config.version
            stream_config.metadata = stream_metadata
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
            ingest_config_dict["metadata"] = ingest_metadata
            ingest_config = IngestConfigWithDestination(**ingest_config_dict)
            self.ingest_api = IngestApi(name, ingest_config, t=self._t)


class EgressConfig(BaseModel):
    """Configuration for Consumption (Egress) APIs.

    Attributes:
        version: Optional version string.
        metadata: Optional metadata for the consumption API.
    """
    version: Optional[str] = None
    metadata: Optional[dict] = None


class ConsumptionApi(BaseTypedResource, Generic[T, U]):
    """Represents a Consumption (Egress) API endpoint.

    Allows querying data, typically powered by a user-defined function.
    Requires two Pydantic models: `T` for query parameters and `U` for the response body.

    Args:
        name: The name of the consumption API endpoint.
        query_function: The callable that executes the query logic.
                      It receives parameters matching model `T` (and potentially
                      other runtime utilities) and should return data matching model `U`.
        config: Optional configuration (currently only `version`).
        t: A tuple containing the input (`T`) and output (`U`) Pydantic models
           (passed via `ConsumptionApi[InputModel, OutputModel](...)`).

    Attributes:
        config (EgressConfig): Configuration for the API.
        query_function (Callable[..., U]): The handler function for the API.
        name (str): The name of the API.
        model_type (type[T]): The Pydantic model for the input/query parameters.
        return_type (type[U]): The Pydantic model for the response body.
    """
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

    def __init__(self, name: str, query_function: Callable[..., U], config: EgressConfig = EgressConfig(), **kwargs):
        super().__init__()
        self._set_type(name, self._get_type(kwargs))
        self.config = config
        self.query_function = query_function
        self.metadata = config.metadata
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

    def return_type(self) -> type[U]:
        """Get the Pydantic model type for the API's response body."""
        return self._u

    def get_response_schema(self) -> JsonSchemaValue:
        """Generates the JSON schema for the API's response body model (`U`).

        Returns:
            A dictionary representing the JSON schema.
        """
        from pydantic.type_adapter import TypeAdapter
        return TypeAdapter(self.return_type).json_schema(
            ref_template='#/components/schemas/{model}'
        )


def _get_consumption_api(name: str) -> Optional[ConsumptionApi]:
    """Internal function to retrieve a registered ConsumptionApi by name."""
    return _egress_apis.get(name)


# Removed BaseModel inheritance
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


class View(SqlResource):
    """Represents a standard SQL database View.

    Args:
        name: The name of the view to be created.
        select_statement: The SQL SELECT statement defining the view.
        base_tables: A list of `OlapTable`, `View`, or `MaterializedView` objects
                     that this view depends on.
    """

    def __init__(self, name: str, select_statement: str, base_tables: list[Union[OlapTable, SqlResource]], metadata: dict = None):
        setup = [
            f"CREATE VIEW IF NOT EXISTS {name} AS {select_statement}".strip()
        ]
        teardown = [f"DROP VIEW IF EXISTS {name}"]
        super().__init__(name, setup, teardown, pulls_data_from=base_tables, metadata=metadata)


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
