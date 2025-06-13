"""
Ingestion Pipeline definitions for Moose Data Model v2 (dmv2).

This module provides classes for defining and configuring complete ingestion pipelines,
which combine tables, streams, and ingestion APIs into a single cohesive unit.
"""
from typing import Any, Optional, Generic, TypeVar
from pydantic import BaseModel

from .types import TypedMooseResource, T
from .olap_table import OlapTable, OlapConfig
from .stream import Stream, StreamConfig, DeadLetterQueue
from .ingest_api import IngestApi, IngestConfig, IngestConfigWithDestination

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
    dead_letter_queue: bool | StreamConfig = True
    version: Optional[str] = None
    metadata: Optional[dict] = None

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
        dead_letter_queue: The created `DeadLetterQueue` instance, if configured.
        columns (Columns[T]): Helper for accessing data field names safely.
        name (str): The base name of the pipeline.
        model_type (type[T]): The Pydantic model associated with this pipeline.
    """
    table: Optional[OlapTable[T]] = None
    stream: Optional[Stream[T]] = None
    ingest_api: Optional[IngestApi[T]] = None
    dead_letter_queue: Optional[DeadLetterQueue[T]] = None
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

    def get_dead_letter_queue(self) -> Stream[T]:
        """Retrieves the pipeline's dead letter queue.

        Raises:
            ValueError: If the dead letter queue was not configured for this pipeline.

        Returns:
            The `Stream` instance.
        """
        if self.dead_letter_queue is None:
            raise ValueError("DLQ was not configured for this pipeline")
        return self.dead_letter_queue

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
        if config.dead_letter_queue:
            stream_config = StreamConfig() if config.dead_letter_queue is True else config.dead_letter_queue
            if config.version:
                stream_config.version = config.version
            stream_config.metadata = stream_metadata
            self.dead_letter_queue = DeadLetterQueue(f"{name}DeadLetterQueue", stream_config, t=self._t)
        if config.ingest:
            if self.stream is None:
                raise ValueError("Ingest API needs a stream to write to.")
            ingest_config_dict = (
                IngestConfig() if config.ingest is True else config.ingest
            ).model_dump()
            ingest_config_dict["destination"] = self.stream
            if config.version:
                ingest_config_dict["version"] = config.version
            if self.dead_letter_queue:
                ingest_config_dict["dead_letter_queue"] = self.dead_letter_queue
            ingest_config_dict["metadata"] = ingest_metadata
            ingest_config = IngestConfigWithDestination(**ingest_config_dict)
            self.ingest_api = IngestApi(name, ingest_config, t=self._t)