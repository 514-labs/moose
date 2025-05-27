/**
 * Shared types and interfaces for the dmv2 module
 */

import { Column } from "../dataModels/dataModelTypes";
import { ClickHouseEngines } from "../index";

/**
 * Configuration options for an OLAP (Online Analytical Processing) table.
 * @template T The data type of the records stored in the table.
 */
export type OlapConfig<T> = {
  /**
   * Specifies the fields to use for ordering data within the ClickHouse table.
   * This is crucial for optimizing query performance, especially for ReplacingMergeTree engines.
   */
  orderByFields?: (keyof T & string)[];
  /**
   * If true, uses the ReplacingMergeTree engine for the ClickHouse table, enabling automatic deduplication based on the `orderByFields`.
   * Equivalent to setting `engine: ClickHouseEngines.ReplacingMergeTree`.
   * Defaults to false.
   */
  // equivalent to setting `engine: ClickHouseEngines.ReplacingMergeTree`
  deduplicate?: boolean;
  /**
   * Specifies the ClickHouse table engine to use.
   * Defaults to MergeTree if not specified.
   * @see ClickHouseEngines for available options.
   */
  engine?: ClickHouseEngines;
  /**
   * An optional version string for this configuration. Can be used for tracking changes or managing deployments.
   */
  version?: string;
};

/**
 * Represents a failed record during insertion with error details
 */
export interface FailedRecord<T> {
  /** The original record that failed to insert */
  record: T;
  /** The error message describing why the insertion failed */
  error: string;
  /** Optional: The index of this record in the original batch */
  index?: number;
}

/**
 * Result of an insert operation with detailed success/failure information
 */
export interface InsertResult<T> {
  /** Number of records successfully inserted */
  successful: number;
  /** Number of records that failed to insert */
  failed: number;
  /** Total number of records processed */
  total: number;
  /** Detailed information about failed records (if record isolation was used) */
  failedRecords?: FailedRecord<T>[];
}

/**
 * Error handling strategy for insert operations
 */
export type ErrorStrategy =
  | "fail-fast" // Fail immediately on any error (default)
  | "discard" // Discard bad records and continue with good ones
  | "isolate"; // Retry individual records to isolate failures

/**
 * Options for insert operations
 */
export interface InsertOptions {
  /** Maximum number of bad records to tolerate before failing */
  allowErrors?: number;
  /** Maximum ratio of bad records to tolerate (0.0 to 1.0) before failing */
  allowErrorsRatio?: number;
  /** Error handling strategy */
  strategy?: ErrorStrategy;
  /** Whether to enable dead letter queue for failed records (future feature) */
  deadLetterQueue?: boolean;
  /** Whether to validate data against schema before insertion (default: true) */
  validate?: boolean;
  /** Whether to skip validation for individual records during 'isolate' strategy retries (default: false) */
  skipValidationOnRetry?: boolean;
}

/**
 * Validation result for a record with detailed error information
 */
export interface ValidationError {
  /** The original record that failed validation */
  record: any;
  /** Detailed validation error message */
  error: string;
  /** Optional: The index of this record in the original batch */
  index?: number;
  /** The path to the field that failed validation */
  path?: string;
}

/**
 * Result of data validation with success/failure breakdown
 */
export interface ValidationResult<T> {
  /** Records that passed validation */
  valid: T[];
  /** Records that failed validation with detailed error information */
  invalid: ValidationError[];
  /** Total number of records processed */
  total: number;
}

/**
 * Configuration options for a data stream (e.g., a Redpanda topic).
 * @template T The data type of the messages in the stream.
 */
export interface StreamConfig<T> {
  /**
   * Specifies the number of partitions for the stream. Affects parallelism and throughput.
   */
  parallelism?: number;
  /**
   * Specifies the data retention period for the stream in seconds. Messages older than this may be deleted.
   */
  retentionPeriod?: number; // seconds
  /**
   * An optional destination OLAP table where messages from this stream should be automatically ingested.
   */
  destination?: any; // OlapTable<T> - avoiding circular dependency
  /**
   * An optional version string for this configuration. Can be used for tracking changes or managing deployments.
   */
  version?: string;
  metadata?: { description?: string };
}

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

export type ZeroOrMany<T> = T | T[] | undefined | null;
export type SyncOrAsyncTransform<T, U> = (
  record: T,
) => ZeroOrMany<U> | Promise<ZeroOrMany<U>>;
export type Consumer<T> = (record: T) => Promise<void> | void;

export interface DeadLetterModel {
  originalRecord: Record<string, any>;
  errorMessage: string;
  errorType: string;
  failedAt: Date;
  source: "api" | "transform" | "table";
}

export interface DeadLetter<T> extends DeadLetterModel {
  asTyped: () => T;
}

export interface TransformConfig<T> {
  version?: string;
  metadata?: { description?: string };
  deadLetterQueue?: any; // DeadLetterQueue<T> - circular dependency, typed as any
}

export interface ConsumerConfig<T> {
  version?: string;
  deadLetterQueue?: any; // DeadLetterQueue<T> - circular dependency, typed as any
}

/**
 * @template T The data type of the messages expected by the destination stream.
 */
export interface IngestConfig<T> {
  /**
   * The destination stream where the ingested data should be sent.
   */
  destination: any; // Stream<T> - avoiding circular dependency
  /**
   * An optional version string for this configuration.
   */
  version?: string;
  metadata?: { description?: string };
}

/**
 * @template T The data type of the request parameters.
 */
export interface EgressConfig<T> {
  /**
   * An optional version string for this configuration.
   */
  version?: string;
  metadata?: { description?: string };
}

/**
 * A helper type used potentially for indicating aggregated fields in query results or schemas.
 * Captures the aggregation function name and argument types.
 * (Usage context might be specific to query builders or ORM features).
 *
 * @template AggregationFunction The name of the aggregation function (e.g., 'sum', 'avg', 'count').
 * @template ArgTypes An array type representing the types of the arguments passed to the aggregation function.
 */
export type Aggregated<
  AggregationFunction extends string,
  ArgTypes extends any[] = [],
> = {
  _aggregationFunction?: AggregationFunction;
  _argTypes?: ArgTypes;
};

export type SqlObject = any; // OlapTable<any> | SqlResource - avoiding circular dependency

// Forward declaration for SqlResource
export interface SqlResource {
  readonly kind: "SqlResource";
  setup: readonly string[];
  teardown: readonly string[];
  name: string;
  pullsDataFrom: SqlObject[];
  pushesDataTo: SqlObject[];
}
