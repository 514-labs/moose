/**
 * IngestPipeline class for creating complete ingestion pipelines
 */

import { Column } from "../dataModels/dataModelTypes";
import { IJsonSchemaCollection } from "typia/src/schemas/json/IJsonSchemaCollection";
import { TypedBase, TypiaValidators } from "./typedBase";
import { OlapConfig, OlapTable } from "./olapTable";
import { Stream, StreamConfig } from "./stream";
import { IngestApi, IngestConfig } from "./api";

/**
 * Configuration options for a complete ingestion pipeline, potentially including an Ingest API, a Stream, and an OLAP Table.
 * @template T The data type of the records being ingested.
 */
export type IngestPipelineConfig<T> = {
  /**
   * Configuration for the OLAP table component of the pipeline.
   * If `true`, a table with default settings is created.
   * If an `OlapConfig` object is provided, it specifies the table's configuration.
   * If `false` no OLAP table is created.
   */
  table: boolean | OlapConfig<T>;
  /**
   * Configuration for the stream component of the pipeline.
   * If `true`, a stream with default settings is created.
   * If a partial `StreamConfig` object (excluding `destination`) is provided, it specifies the stream's configuration.
   * The stream's destination will automatically be set to the pipeline's table if one exists.
   * If `false`, no stream is created.
   */
  stream: boolean | Omit<StreamConfig<T>, "destination">;
  /**
   * Configuration for the ingest API component of the pipeline.
   * If `true`, an ingest API with default settings is created.
   * If a partial `IngestConfig` object (excluding `destination`) is provided, it specifies the API's configuration.
   * The API's destination will automatically be set to the pipeline's stream if one exists. Requires a stream to be configured.
   * If `false`, no ingest API is created.
   */
  ingest: boolean | Omit<IngestConfig<T>, "destination">;
  /**
   * An optional version string applying to all components (table, stream, ingest) created by this pipeline configuration.
   */
  version?: string;
  metadata?: { description?: string };
};

/**
 * Represents a complete ingestion pipeline, potentially combining an Ingest API, a Stream, and an Olap Table
 * under a single name and configuration. Simplifies the setup of common ingestion patterns.
 *
 * @template T The data type of the records flowing through the pipeline. This type defines the schema for the
 *             Ingest API input, the Stream messages, and the Olap Table rows.
 */
export class IngestPipeline<T> extends TypedBase<T, IngestPipelineConfig<T>> {
  metadata?: { description?: string };
  /** The OLAP table component of the pipeline, if configured. */
  table?: OlapTable<T>;
  /** The stream component of the pipeline, if configured. */
  stream?: Stream<T>;
  /** The ingest API component of the pipeline, if configured. */
  ingestApi?: IngestApi<T>;

  /**
   * Creates a new IngestPipeline instance.
   * Based on the configuration, it automatically creates and links the IngestApi, Stream, and OlapTable components.
   *
   * @param name The base name for the pipeline components (e.g., "userData" could create "userData" table, "userData" stream, "userData" ingest API).
   * @param config Configuration specifying which components (table, stream, ingest) to create and their settings.
   */
  constructor(name: string, config: IngestPipelineConfig<T>);

  /** @internal **/
  constructor(
    name: string,
    config: IngestPipelineConfig<T>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
    validators?: TypiaValidators<T>,
  );

  constructor(
    name: string,
    config: IngestPipelineConfig<T>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
    validators?: TypiaValidators<T>,
  ) {
    super(name, config, schema, columns, validators);
    this.metadata = config?.metadata;

    if (config.table) {
      const tableConfig = {
        ...(typeof config.table === "object" ? config.table : {}),
        ...(config.version && { version: config.version }),
      };
      this.table = new OlapTable(
        name,
        tableConfig,
        this.schema,
        this.columnArray,
        this.validators,
      );
    }

    if (config.stream) {
      const streamConfig = {
        destination: this.table,
        ...(typeof config.stream === "object" ? config.stream : {}),
        ...(config.version && { version: config.version }),
      };
      this.stream = new Stream(
        name,
        streamConfig,
        this.schema,
        this.columnArray,
      );
      (this.stream as any).pipelineParent = this;
    }

    if (config.ingest) {
      if (!this.stream) {
        throw new Error("Ingest API needs a stream to write to.");
      }

      const ingestConfig = {
        destination: this.stream,
        ...(typeof config.ingest === "object" ? config.ingest : {}),
        ...(config.version && { version: config.version }),
      };
      this.ingestApi = new IngestApi(
        name,
        ingestConfig,
        this.schema,
        this.columnArray,
      );
      (this.ingestApi as any).pipelineParent = this;
    }
  }
}
