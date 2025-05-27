/**
 * @module dmv2
 * This module defines the core Moose v2 data model constructs, including OlapTable, Stream, IngestApi, ConsumptionApi,
 * IngestPipeline, View, and MaterializedView. These classes provide a typed interface for defining and managing
 * data infrastructure components like ClickHouse tables, Redpanda streams, and data processing pipelines.
 */
import { Column } from "../dataModels/dataModelTypes";
import { IJsonSchemaCollection } from "typia/src/schemas/json/IJsonSchemaCollection";
import { getMooseInternal } from "./internal";
import { TypedBase, TypiaValidators } from "./typedBase";
import {
  ClickHouseEngines,
  ConsumptionUtil,
  createMaterializedView,
  dropView,
  populateTable,
  Sql,
  toQuery,
  toStaticQuery,
} from "../index";
import { Readable } from "stream";

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
  destination?: OlapTable<T>;
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

/**
 * Represents an OLAP (Online Analytical Processing) table, typically corresponding to a ClickHouse table.
 * Provides a typed interface for interacting with the table.
 *
 * @template T The data type of the records stored in the table. The structure of T defines the table schema.
 */
export class OlapTable<T> extends TypedBase<T, OlapConfig<T>> {
  /** @internal */
  public readonly kind = "OlapTable";

  /** @internal Memoized ClickHouse client for reusing connections across insert calls */
  private _memoizedClient?: any;
  /** @internal Hash of the configuration used to create the memoized client */
  private _configHash?: string;

  /**
   * Creates a new OlapTable instance.
   * @param name The name of the table. This name is used for the underlying ClickHouse table.
   * @param config Optional configuration for the OLAP table.
   */
  constructor(name: string, config?: OlapConfig<T>);

  /** @internal **/
  constructor(
    name: string,
    config: OlapConfig<T>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
    validators?: TypiaValidators<T>,
  );

  constructor(
    name: string,
    config?: OlapConfig<T>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
    validators?: TypiaValidators<T>,
  ) {
    super(name, config ?? {}, schema, columns, validators);

    getMooseInternal().tables.set(name, this);
  }

  /**
   * Generates the versioned table name following Moose's naming convention
   * Format: {tableName}_{version_with_dots_replaced_by_underscores}
   */
  generateTableName(): string {
    const tableVersion = this.config.version;
    if (!tableVersion) {
      return this.name;
    } else {
      const versionSuffix = tableVersion.replace(/\./g, "_");
      return `${this.name}_${versionSuffix}`;
    }
  }

  /**
   * Gets or creates a memoized ClickHouse client.
   * The client is cached and reused across multiple insert calls for better performance.
   * If the configuration changes, a new client will be created.
   *
   * @private
   */
  private async getMemoizedClient() {
    const { readProjectConfig } = await import("../config");
    const { getClickhouseClient } = await import("../commons");

    // Read current configuration
    const config = readProjectConfig();
    const clickhouseConfig = config.clickhouse_config;

    // Create a hash of the current configuration to detect changes
    const currentConfigHash = JSON.stringify({
      host: clickhouseConfig.host,
      port: clickhouseConfig.host_port,
      user: clickhouseConfig.user,
      password: clickhouseConfig.password,
      database: clickhouseConfig.db_name,
      useSSL: clickhouseConfig.use_ssl,
    });

    // If we have a cached client and the config hasn't changed, reuse it
    if (this._memoizedClient && this._configHash === currentConfigHash) {
      return { client: this._memoizedClient, config: clickhouseConfig };
    }

    // Close existing client if config changed
    if (this._memoizedClient && this._configHash !== currentConfigHash) {
      try {
        await this._memoizedClient.close();
      } catch (error) {
        // Ignore errors when closing old client
      }
    }

    // Create new client
    const client = getClickhouseClient({
      username: clickhouseConfig.user,
      password: clickhouseConfig.password,
      database: clickhouseConfig.db_name,
      useSSL: clickhouseConfig.use_ssl ? "true" : "false",
      host: clickhouseConfig.host,
      port: clickhouseConfig.host_port.toString(),
    });

    // Cache the new client and config hash
    this._memoizedClient = client;
    this._configHash = currentConfigHash;

    return { client, config: clickhouseConfig };
  }

  /**
   * Closes the memoized ClickHouse client if it exists.
   * This is useful for cleaning up connections when the table instance is no longer needed.
   * The client will be automatically recreated on the next insert call if needed.
   */
  async closeClient(): Promise<void> {
    if (this._memoizedClient) {
      try {
        await this._memoizedClient.close();
      } catch (error) {
        // Ignore errors when closing
      } finally {
        this._memoizedClient = undefined;
        this._configHash = undefined;
      }
    }
  }

  /**
   * Validates a single record using typia's comprehensive type checking.
   * This provides the most accurate validation as it uses the exact TypeScript type information.
   *
   * @param record The record to validate
   * @returns Validation result with detailed error information
   */
  validateRecord(record: unknown): {
    success: boolean;
    data?: T;
    errors?: string[];
  } {
    // Use injected typia validator if available
    if (this.validators?.validate) {
      try {
        const result = this.validators.validate(record);
        return {
          success: result.success,
          data: result.data,
          errors: result.errors?.map((err) =>
            typeof err === "string" ? err : JSON.stringify(err),
          ),
        };
      } catch (error) {
        return {
          success: false,
          errors: [error instanceof Error ? error.message : String(error)],
        };
      }
    }

    // Fallback to basic validation if typia validators aren't available
    return this.basicValidateRecord(record);
  }

  /**
   * Basic validation fallback that doesn't require typia injection.
   * Performs fundamental checks like null/undefined and required fields.
   *
   * @param record The record to validate
   * @returns Basic validation result
   */
  private basicValidateRecord(record: unknown): {
    success: boolean;
    data?: T;
    errors?: string[];
  } {
    if (record === null || record === undefined) {
      return {
        success: false,
        errors: ["Record cannot be null or undefined"],
      };
    }

    if (typeof record !== "object") {
      return {
        success: false,
        errors: ["Record must be an object"],
      };
    }

    // Check required fields based on column metadata
    const errors: string[] = [];
    const data = record as Record<string, any>;

    for (const column of this.columnArray) {
      if (
        column.required &&
        (data[column.name] === undefined || data[column.name] === null)
      ) {
        errors.push(`Required field '${column.name}' is missing`);
      }
    }

    return errors.length > 0 ?
        { success: false, errors }
      : { success: true, data: record as T };
  }

  /**
   * Type guard function using typia's is() function.
   * Provides compile-time type narrowing for TypeScript.
   *
   * @param record The record to check
   * @returns True if record matches type T, with type narrowing
   */
  isValidRecord(record: unknown): record is T {
    if (this.validators?.is) {
      return this.validators.is(record);
    }

    // Fallback: basic type check
    const result = this.basicValidateRecord(record);
    return result.success;
  }

  /**
   * Assert that a record matches type T, throwing detailed errors if not.
   * Uses typia's assert() function for the most detailed error reporting.
   *
   * @param record The record to assert
   * @returns The validated and typed record
   * @throws Detailed validation error if record doesn't match type T
   */
  assertValidRecord(record: unknown): T {
    if (this.validators?.assert) {
      return this.validators.assert(record);
    }

    // Fallback: basic assertion
    const result = this.basicValidateRecord(record);
    if (!result.success) {
      throw new Error(`Validation failed: ${result.errors?.join(", ")}`);
    }
    return result.data!;
  }

  /**
   * Validates an array of records with comprehensive error reporting.
   * Uses the most appropriate validation method available (typia or basic).
   *
   * @param data Array of records to validate
   * @returns Detailed validation results
   */
  async validateRecords(data: unknown[]): Promise<ValidationResult<T>> {
    const valid: T[] = [];
    const invalid: ValidationError[] = [];

    for (let i = 0; i < data.length; i++) {
      const record = data[i];
      const result = this.validateRecord(record);

      if (result.success && result.data) {
        valid.push(result.data);
      } else {
        invalid.push({
          record,
          error: result.errors?.join(", ") || "Validation failed",
          index: i,
          path: "root",
        });
      }
    }

    return {
      valid,
      invalid,
      total: data.length,
    };
  }

  /**
   * Retries individual records from a failed batch to isolate which ones are causing errors.
   * This provides detailed error information for each failed record.
   *
   * @private
   */
  private async retryIndividualRecords(
    client: any,
    tableName: string,
    records: T[],
  ): Promise<{ successful: T[]; failed: FailedRecord<T>[] }> {
    const successful: T[] = [];
    const failed: FailedRecord<T>[] = [];

    // Process records individually to isolate failures
    for (let i = 0; i < records.length; i++) {
      const record = records[i];
      try {
        await client.insert({
          table: tableName,
          values: [record],
          format: "JSONEachRow",
          clickhouse_settings: {
            // Allows to insert serialized JS Dates (such as '2023-12-06T10:54:48.000Z')
            date_time_input_format: "best_effort",
          },
        });
        successful.push(record);
      } catch (error) {
        failed.push({
          record,
          error: error instanceof Error ? error.message : String(error),
          index: i,
        });
      }
    }

    return { successful, failed };
  }

  /**
   * Inserts data directly into the ClickHouse table with enhanced error handling and validation.
   * This method establishes a direct connection to ClickHouse using the project configuration
   * and inserts the provided data into the versioned table.
   *
   * Uses advanced typia validation when available for comprehensive type checking,
   * with fallback to basic validation for compatibility.
   *
   * The ClickHouse client is memoized and reused across multiple insert calls for better performance.
   * If the configuration changes, a new client will be automatically created.
   *
   * @param data Array of objects conforming to the table schema, or a Node.js Readable stream
   * @param options Optional configuration for error handling, validation, and insertion behavior
   * @returns Promise resolving to detailed insertion results
   * @throws {ConfigError} When configuration cannot be read or parsed
   * @throws {ClickHouseError} When insertion fails based on the error strategy
   * @throws {ValidationError} When validation fails and strategy is 'fail-fast'
   *
   * @example
   * ```typescript
   * // Create an OlapTable instance (typia validators auto-injected)
   * const userTable = new OlapTable<User>('users');
   *
   * // Insert with comprehensive typia validation
   * const result1 = await userTable.insert([
   *   { id: 1, name: 'John', email: 'john@example.com' },
   *   { id: 2, name: 'Jane', email: 'jane@example.com' }
   * ]);
   *
   * // Insert data with stream input (validation not available for streams)
   * const dataStream = new Readable({
   *   objectMode: true,
   *   read() { // Stream implementation }
   * });
   * const result2 = await userTable.insert(dataStream, { strategy: 'fail-fast' });
   *
   * // Insert with validation disabled for performance
   * const result3 = await userTable.insert(data, { validate: false });
   *
   * // Insert with error handling strategies
   * const result4 = await userTable.insert(mixedData, {
   *   strategy: 'isolate',
   *   allowErrorsRatio: 0.1,
   *   validate: true  // Use typia validation (default)
   * });
   *
   * // Optional: Clean up connection when completely done
   * await userTable.closeClient();
   * ```
   */
  async insert(
    data: T[] | Readable,
    options?: InsertOptions,
  ): Promise<InsertResult<T>> {
    const isStream = data instanceof Readable;
    const strategy = options?.strategy || "fail-fast";
    const shouldValidate = options?.validate !== false; // Default to true
    const skipValidationOnRetry = options?.skipValidationOnRetry || false;

    // Validate strategy compatibility with streams
    if (isStream && strategy === "isolate") {
      throw new Error(
        "The 'isolate' error strategy is not supported with stream input. Use 'fail-fast' or 'discard' instead.",
      );
    }

    // Validate that validation is not attempted on streams
    if (isStream && shouldValidate) {
      console.warn(
        "Validation is not supported with stream input. Validation will be skipped.",
      );
    }

    if (isStream && !data) {
      return {
        successful: 0,
        failed: 0,
        total: 0,
      };
    }

    if (!isStream && (!data || data.length === 0)) {
      return {
        successful: 0,
        failed: 0,
        total: 0,
      };
    }

    // Pre-insertion validation for arrays
    let validatedData: T[] = [];
    let validationErrors: ValidationError[] = [];

    if (!isStream && shouldValidate) {
      try {
        const validationResult = await this.validateRecords(data as unknown[]);
        validatedData = validationResult.valid;
        validationErrors = validationResult.invalid;

        // Handle validation errors based on strategy
        if (validationErrors.length > 0) {
          switch (strategy) {
            case "fail-fast":
              const firstError = validationErrors[0];
              throw new Error(
                `Validation failed for record at index ${firstError.index}: ${firstError.error}`,
              );

            case "discard":
              // Check if validation errors exceed thresholds
              const validationFailedCount = validationErrors.length;
              const validationFailedRatio =
                validationFailedCount / (data as T[]).length;

              if (
                options?.allowErrors !== undefined &&
                validationFailedCount > options.allowErrors
              ) {
                throw new Error(
                  `Too many validation failures: ${validationFailedCount} > ${options.allowErrors}. Errors: ${validationErrors.map((e) => e.error).join(", ")}`,
                );
              }

              if (
                options?.allowErrorsRatio !== undefined &&
                validationFailedRatio > options.allowErrorsRatio
              ) {
                throw new Error(
                  `Validation failure ratio too high: ${validationFailedRatio.toFixed(3)} > ${options.allowErrorsRatio}. Errors: ${validationErrors.map((e) => e.error).join(", ")}`,
                );
              }

              // Continue with only valid data
              break;

            case "isolate":
              // For isolate strategy, we'll handle validation errors in the final result
              // Continue with all data (valid + invalid) and let ClickHouse handle the invalid ones
              validatedData = data as T[];
              break;
          }
        }
      } catch (validationError) {
        if (strategy === "fail-fast") {
          throw validationError;
        }
        // For other strategies, log the validation error but continue
        console.warn("Validation error:", validationError);
        validatedData = data as T[];
      }
    } else {
      // No validation or stream input
      validatedData = isStream ? [] : (data as T[]);
    }

    // Get memoized client and current config
    const { client, config: clickhouseConfig } = await this.getMemoizedClient();

    // Generate the versioned table name
    const tableName = this.generateTableName();

    try {
      // Prepare insert options
      const insertOptions: any = {
        table: tableName,
        format: "JSONEachRow",
        clickhouse_settings: {
          // Allows to insert serialized JS Dates (such as '2023-12-06T10:54:48.000Z')
          date_time_input_format: "best_effort",
        },
      };

      // Handle stream vs array input
      if (isStream) {
        insertOptions.values = data; // ClickHouse client accepts streams in the values field
      } else {
        insertOptions.values = validatedData;
      }

      // For discard strategy, add ClickHouse error tolerance settings
      if (
        strategy === "discard" &&
        (options?.allowErrors !== undefined ||
          options?.allowErrorsRatio !== undefined)
      ) {
        const querySettings: Record<string, any> = {};

        if (options.allowErrors !== undefined) {
          querySettings.input_format_allow_errors_num = options.allowErrors;
        }

        if (options.allowErrorsRatio !== undefined) {
          querySettings.input_format_allow_errors_ratio =
            options.allowErrorsRatio;
        }

        insertOptions.query_params = querySettings;
      }

      // Try insertion
      await client.insert(insertOptions);

      // For streams, we can't easily count records, so we return success with unknown count
      if (isStream) {
        return {
          successful: -1, // -1 indicates stream mode where count is unknown
          failed: 0,
          total: -1,
        };
      } else {
        // Include validation results in the response
        const insertedCount = validatedData.length;
        const totalProcessed =
          shouldValidate ? (data as T[]).length : insertedCount;

        const result: InsertResult<T> = {
          successful: insertedCount,
          failed: shouldValidate ? validationErrors.length : 0,
          total: totalProcessed,
        };

        // Add failed records if there are validation errors and using discard strategy
        if (
          shouldValidate &&
          validationErrors.length > 0 &&
          strategy === "discard"
        ) {
          result.failedRecords = validationErrors.map((ve) => ({
            record: ve.record as T,
            error: `Validation error: ${ve.error}`,
            index: ve.index,
          }));
        }

        return result;
      }
    } catch (batchError) {
      // Handle insertion failure based on strategy
      switch (strategy) {
        case "fail-fast":
          throw new Error(
            `Failed to insert data into table ${tableName}: ${batchError}`,
          );

        case "discard":
          // With discard strategy and ClickHouse error tolerance, this shouldn't happen
          // unless the error threshold was exceeded
          throw new Error(
            `Too many errors during insert into table ${tableName}. Error threshold exceeded: ${batchError}`,
          );

        case "isolate":
          // Only supported for arrays, not streams
          if (isStream) {
            throw new Error(
              `Isolate strategy is not supported with stream input: ${batchError}`,
            );
          }

          // Retry individual records to isolate failures and provide detailed error information
          try {
            const retryData =
              skipValidationOnRetry ? (data as T[]) : validatedData;
            const { successful, failed } = await this.retryIndividualRecords(
              client,
              tableName,
              retryData,
            );

            // Combine validation errors with insertion errors
            const allFailedRecords: FailedRecord<T>[] = [
              // Validation errors (if any and not skipping validation on retry)
              ...(shouldValidate && !skipValidationOnRetry ?
                validationErrors.map((ve) => ({
                  record: ve.record as T,
                  error: `Validation error: ${ve.error}`,
                  index: ve.index,
                }))
              : []),
              // Insertion errors
              ...failed,
            ];

            const totalFailed = allFailedRecords.length;
            const totalProcessed = (data as T[]).length;
            const failedRatio = totalFailed / totalProcessed;

            // Check if we exceeded error thresholds
            if (
              options?.allowErrors !== undefined &&
              totalFailed > options.allowErrors
            ) {
              throw new Error(
                `Too many failed records: ${totalFailed} > ${options.allowErrors}. Failed records: ${allFailedRecords.map((f) => f.error).join(", ")}`,
              );
            }

            if (
              options?.allowErrorsRatio !== undefined &&
              failedRatio > options.allowErrorsRatio
            ) {
              throw new Error(
                `Failed record ratio too high: ${failedRatio.toFixed(3)} > ${options.allowErrorsRatio}. Failed records: ${allFailedRecords.map((f) => f.error).join(", ")}`,
              );
            }

            // Return detailed results
            return {
              successful: successful.length,
              failed: totalFailed,
              total: totalProcessed,
              failedRecords: allFailedRecords,
            };
          } catch (isolationError) {
            throw new Error(
              `Failed to insert data into table ${tableName} during record isolation: ${isolationError}`,
            );
          }

        default:
          throw new Error(`Unknown error strategy: ${strategy}`);
      }
    }
    // Note: We don't close the client here since it's memoized for reuse
    // Use closeClient() method if you need to explicitly close the connection
  }
}

type ZeroOrMany<T> = T | T[] | undefined | null;
type SyncOrAsyncTransform<T, U> = (
  record: T,
) => ZeroOrMany<U> | Promise<ZeroOrMany<U>>;
type Consumer<T> = (record: T) => Promise<void> | void;

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

function attachTypeGuard<T>(
  dl: DeadLetterModel,
  typeGuard: (input: any) => T,
): asserts dl is DeadLetter<T> {
  (dl as any).asTyped = () => typeGuard(dl.originalRecord);
}

export interface TransformConfig<T> {
  version?: string;
  metadata?: { description?: string };
  deadLetterQueue?: DeadLetterQueue<T>;
}

export interface ConsumerConfig<T> {
  version?: string;
  deadLetterQueue?: DeadLetterQueue<T>;
}

/**
 * Represents a data stream, typically corresponding to a Redpanda topic.
 * Provides a typed interface for producing to and consuming from the stream, and defining transformations.
 *
 * @template T The data type of the messages flowing through the stream. The structure of T defines the message schema.
 */
export class Stream<T> extends TypedBase<T, StreamConfig<T>> {
  metadata?: { description?: string };
  /**
   * Creates a new Stream instance.
   * @param name The name of the stream. This name is used for the underlying Redpanda topic.
   * @param config Optional configuration for the stream.
   */
  constructor(name: string, config?: StreamConfig<T>);

  /** @internal **/
  constructor(
    name: string,
    config: StreamConfig<T>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
  );

  constructor(
    name: string,
    config?: StreamConfig<T>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
  ) {
    super(name, config ?? {}, schema, columns);
    this.metadata = config?.metadata;
    getMooseInternal().streams.set(name, this);
  }

  _transformations = new Map<
    string,
    [Stream<any>, SyncOrAsyncTransform<T, any>, TransformConfig<T>][]
  >();
  _multipleTransformations?: (record: T) => [RoutedMessage];
  _consumers = new Array<{
    consumer: Consumer<T>;
    config: ConsumerConfig<T>;
  }>();

  /**
   * Adds a transformation step that processes messages from this stream and sends the results to a destination stream.
   * Multiple transformations to the same destination stream can be added if they have distinct `version` identifiers in their config.
   *
   * @template U The data type of the messages in the destination stream.
   * @param destination The destination stream for the transformed messages.
   * @param transformation A function that takes a message of type T and returns zero or more messages of type U (or a Promise thereof).
   *                       Return `null` or `undefined` or an empty array `[]` to filter out a message. Return an array to emit multiple messages.
   * @param config Optional configuration for this specific transformation step, like a version.
   */
  addTransform<U>(
    destination: Stream<U>,
    transformation: SyncOrAsyncTransform<T, U>,
    config?: TransformConfig<T>,
  ) {
    const transformConfig = config ?? {};

    if (this._transformations.has(destination.name)) {
      const existingTransforms = this._transformations.get(destination.name)!;
      const hasVersion = existingTransforms.some(
        ([_, __, cfg]) => cfg.version === transformConfig.version,
      );

      if (!hasVersion) {
        existingTransforms.push([destination, transformation, transformConfig]);
      }
    } else {
      this._transformations.set(destination.name, [
        [destination, transformation, transformConfig],
      ]);
    }
  }

  /**
   * Adds a consumer function that processes messages from this stream.
   * Multiple consumers can be added if they have distinct `version` identifiers in their config.
   *
   * @param consumer A function that takes a message of type T and performs an action (e.g., side effect, logging). Should return void or Promise<void>.
   * @param config Optional configuration for this specific consumer, like a version.
   */
  addConsumer(consumer: Consumer<T>, config?: ConsumerConfig<T>) {
    const consumerConfig = config ?? {};
    const hasVersion = this._consumers.some(
      (existing) => existing.config.version === consumerConfig.version,
    );

    if (!hasVersion) {
      this._consumers.push({ consumer, config: consumerConfig });
    }
  }

  /**
   * Helper method for `addMultiTransform` to specify the destination and values for a routed message.
   * @param values The value or values to send to this stream.
   * @returns A `RoutedMessage` object associating the values with this stream.
   */
  routed = (values: ZeroOrMany<T>) => new RoutedMessage(this, values);

  /**
   * Adds a single transformation function that can route messages to multiple destination streams.
   * This is an alternative to adding multiple individual `addTransform` calls.
   * Only one multi-transform function can be added per stream.
   *
   * @param transformation A function that takes a message of type T and returns an array of `RoutedMessage` objects,
   *                       each specifying a destination stream and the message(s) to send to it.
   */
  addMultiTransform(transformation: (record: T) => [RoutedMessage]) {
    this._multipleTransformations = transformation;
  }
}

export class DeadLetterQueue<T> extends Stream<DeadLetterModel> {
  constructor(name: string, config?: StreamConfig<DeadLetterModel>);

  /** @internal **/
  constructor(
    name: string,
    config: StreamConfig<DeadLetterModel>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
    validate: (originalRecord: any) => T,
  );

  constructor(
    name: string,
    config?: StreamConfig<DeadLetterModel>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
    typeGuard?: (originalRecord: any) => T,
  ) {
    if (
      schema === undefined ||
      columns === undefined ||
      typeGuard === undefined
    ) {
      throw new Error(
        "Supply the type param T so that the schema is inserted by the compiler plugin.",
      );
    }

    super(name, config ?? {}, schema, columns);
    this.typeGuard = typeGuard;
    getMooseInternal().streams.set(name, this);
  }
  private typeGuard: (originalRecord: any) => T;

  addTransform<U>(
    destination: Stream<U>,
    transformation: SyncOrAsyncTransform<DeadLetter<T>, U>,
    config?: TransformConfig<DeadLetterModel>,
  ) {
    const withValidate: SyncOrAsyncTransform<DeadLetterModel, U> = (
      deadLetter,
    ) => {
      attachTypeGuard<T>(deadLetter, this.typeGuard);
      return transformation(deadLetter);
    };
    super.addTransform(destination, withValidate, config);
  }
  addConsumer(
    consumer: Consumer<DeadLetter<T>>,
    config?: ConsumerConfig<DeadLetterModel>,
  ) {
    const withValidate: Consumer<DeadLetterModel> = (deadLetter) => {
      attachTypeGuard<T>(deadLetter, this.typeGuard);
      return consumer(deadLetter);
    };
    super.addConsumer(withValidate, config);
  }

  addMultiTransform(
    transformation: (record: DeadLetter<T>) => [RoutedMessage],
  ) {
    const withValidate: (record: DeadLetterModel) => [RoutedMessage] = (
      deadLetter,
    ) => {
      attachTypeGuard<T>(deadLetter, this.typeGuard);
      return transformation(deadLetter);
    };
    super.addMultiTransform(withValidate);
  }
}

class RoutedMessage {
  destination: Stream<any>;
  values: ZeroOrMany<any>;
  constructor(destination: Stream<any>, values: ZeroOrMany<any>) {
    this.destination = destination;
    this.values = values;
  }
}

/**
 * @template T The data type of the messages expected by the destination stream.
 */
interface IngestConfig<T> {
  /**
   * The destination stream where the ingested data should be sent.
   */
  destination: Stream<T>;
  /**
   * An optional version string for this configuration.
   */
  version?: string;
  metadata?: { description?: string };
}

/**
 * Represents an Ingest API endpoint, used for sending data into a Moose system, typically writing to a Stream.
 * Provides a typed interface for the expected data format.
 *
 * @template T The data type of the records that this API endpoint accepts. The structure of T defines the expected request body schema.
 */
export class IngestApi<T> extends TypedBase<T, IngestConfig<T>> {
  metadata?: { description?: string };
  /**
   * Creates a new IngestApi instance.
   * @param name The name of the ingest API endpoint.
   * @param config Configuration for the ingest API, including the destination stream.
   */
  constructor(name: string, config?: IngestConfig<T>);

  /** @internal **/
  constructor(
    name: string,
    config: IngestConfig<T>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
  );

  constructor(
    name: string,
    config: IngestConfig<T>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
  ) {
    super(name, config, schema, columns);
    this.metadata = config?.metadata;
    getMooseInternal().ingestApis.set(name, this);
  }
}

/**
 * Defines the signature for a handler function used by a Consumption API.
 * @template T The expected type of the request parameters or query parameters.
 * @template R The expected type of the response data.
 * @param params An object containing the validated request parameters, matching the structure of T.
 * @param utils Utility functions provided to the handler, e.g., for database access (`runSql`).
 * @returns A Promise resolving to the response data of type R.
 */
type ConsumptionHandler<T, R> = (
  params: T,
  utils: ConsumptionUtil,
) => Promise<R>;

/**
 * @template T The data type of the request parameters.
 */
interface EgressConfig<T> {
  /**
   * An optional version string for this configuration.
   */
  version?: string;
  metadata?: { description?: string };
}

/**
 * Represents a Consumption API endpoint (Egress API), used for querying data from a Moose system.
 * Exposes data, often from an OlapTable or derived through a custom handler function.
 *
 * @template T The data type defining the expected structure of the API's query parameters.
 * @template R The data type defining the expected structure of the API's response body. Defaults to `any`.
 */
export class ConsumptionApi<T, R = any> extends TypedBase<T, EgressConfig<T>> {
  metadata?: { description?: string };
  /** @internal The handler function that processes requests and generates responses. */
  _handler: ConsumptionHandler<T, R>;
  /** @internal The JSON schema definition for the response type R. */
  responseSchema: IJsonSchemaCollection.IV3_1;

  /**
   * Creates a new ConsumptionApi instance.
   * @param name The name of the consumption API endpoint.
   * @param handler The function to execute when the endpoint is called. It receives validated query parameters and utility functions.
   * @param config Optional configuration for the consumption API.
   */
  constructor(name: string, handler: ConsumptionHandler<T, R>, config?: {});

  /** @internal **/
  constructor(
    name: string,
    handler: ConsumptionHandler<T, R>,
    config: EgressConfig<T>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
    responseSchema: IJsonSchemaCollection.IV3_1,
  );

  constructor(
    name: string,
    handler: ConsumptionHandler<T, R>,
    config?: EgressConfig<T>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
    responseSchema?: IJsonSchemaCollection.IV3_1,
  ) {
    super(name, config ?? {}, schema, columns);
    this.metadata = config?.metadata;
    this._handler = handler;
    this.responseSchema = responseSchema ?? {
      version: "3.1",
      schemas: [{ type: "array", items: { type: "object" } }],
      components: { schemas: {} },
    };
    getMooseInternal().egressApis.set(name, this);
  }

  /**
   * Retrieves the handler function associated with this Consumption API.
   * @returns The handler function.
   */
  getHandler = (): ConsumptionHandler<T, R> => {
    return this._handler;
  };
}

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

type SqlObject = OlapTable<any> | SqlResource;

/**
 * Represents a generic SQL resource that requires setup and teardown commands.
 * Base class for constructs like Views and Materialized Views. Tracks dependencies.
 */
export class SqlResource {
  /** @internal */
  public readonly kind = "SqlResource";

  /** Array of SQL statements to execute for setting up the resource. */
  setup: readonly string[];
  /** Array of SQL statements to execute for tearing down the resource. */
  teardown: readonly string[];
  /** The name of the SQL resource (e.g., view name, materialized view name). */
  name: string;

  /** List of OlapTables or Views that this resource reads data from. */
  pullsDataFrom: SqlObject[];
  /** List of OlapTables or Views that this resource writes data to. */
  pushesDataTo: SqlObject[];

  /**
   * Creates a new SqlResource instance.
   * @param name The name of the resource.
   * @param setup An array of SQL DDL statements to create the resource.
   * @param teardown An array of SQL DDL statements to drop the resource.
   * @param options Optional configuration for specifying data dependencies.
   * @param options.pullsDataFrom Tables/Views this resource reads from.
   * @param options.pushesDataTo Tables/Views this resource writes to.
   */
  constructor(
    name: string,
    setup: readonly string[],
    teardown: readonly string[],
    options?: {
      pullsDataFrom?: SqlObject[];
      pushesDataTo?: SqlObject[];
    },
  ) {
    getMooseInternal().sqlResources.set(name, this);

    this.name = name;
    this.setup = setup;
    this.teardown = teardown;
    this.pullsDataFrom = options?.pullsDataFrom ?? [];
    this.pushesDataTo = options?.pushesDataTo ?? [];
  }
}

/**
 * Represents a database View, defined by a SQL SELECT statement based on one or more base tables or other views.
 * Inherits from SqlResource, providing setup (CREATE VIEW) and teardown (DROP VIEW) commands.
 */
export class View extends SqlResource {
  /**
   * Creates a new View instance.
   * @param name The name of the view to be created.
   * @param selectStatement The SQL SELECT statement that defines the view's logic.
   * @param baseTables An array of OlapTable or View objects that the `selectStatement` reads from. Used for dependency tracking.
   */
  constructor(
    name: string,
    selectStatement: string | Sql,
    baseTables: (OlapTable<any> | View)[],
  ) {
    if (typeof selectStatement !== "string") {
      selectStatement = toStaticQuery(selectStatement);
    }

    super(
      name,
      [
        `CREATE VIEW IF NOT EXISTS ${name} 
        AS ${selectStatement}`.trim(),
      ],
      [dropView(name)],
      {
        pullsDataFrom: baseTables,
      },
    );
  }
}

/**
 * Configuration options for creating a Materialized View.
 * @template T The data type of the records stored in the target table of the materialized view.
 */
interface MaterializedViewOptions<T> {
  /** The SQL SELECT statement or `Sql` object defining the data to be materialized. Dynamic SQL (with parameters) is not allowed here. */
  selectStatement: string | Sql;
  /** An array of OlapTable or View objects that the `selectStatement` reads from. */
  selectTables: (OlapTable<any> | View)[];

  /** The name for the underlying target OlapTable that stores the materialized data. */
  tableName: string;
  /** The name for the ClickHouse MATERIALIZED VIEW object itself. */
  materializedViewName: string;

  /** Optional ClickHouse engine for the target table (e.g., ReplacingMergeTree). Defaults to MergeTree. */
  engine?: ClickHouseEngines;
  /** Optional ordering fields for the target table. Crucial if using ReplacingMergeTree. */
  orderByFields?: (keyof T & string)[];
}

/**
 * Represents a Materialized View in ClickHouse.
 * This encapsulates both the target OlapTable that stores the data and the MATERIALIZED VIEW definition
 * that populates the table based on inserts into the source tables.
 *
 * @template TargetTable The data type of the records stored in the underlying target OlapTable. The structure of T defines the target table schema.
 */
export class MaterializedView<TargetTable> extends SqlResource {
  /** The target OlapTable instance where the materialized data is stored. */
  targetTable: OlapTable<TargetTable>;

  /**
   * Creates a new MaterializedView instance.
   * Requires the `TargetTable` type parameter to be explicitly provided or inferred,
   * as it's needed to define the schema of the underlying target table.
   *
   * @param options Configuration options for the materialized view.
   */
  constructor(options: MaterializedViewOptions<TargetTable>);

  /** @internal **/
  constructor(
    options: MaterializedViewOptions<TargetTable>,
    targetSchema: IJsonSchemaCollection.IV3_1,
    targetColumns: Column[],
  );
  constructor(
    options: MaterializedViewOptions<TargetTable>,
    targetSchema?: IJsonSchemaCollection.IV3_1,
    targetColumns?: Column[],
  ) {
    let selectStatement = options.selectStatement;
    if (typeof selectStatement !== "string") {
      selectStatement = toStaticQuery(selectStatement);
    }

    if (targetSchema === undefined || targetColumns === undefined) {
      throw new Error(
        "Supply the type param T so that the schema is inserted by the compiler plugin.",
      );
    }

    const targetTable = new OlapTable(
      options.tableName,
      {
        orderByFields: options.orderByFields,
      },
      targetSchema,
      targetColumns,
    );
    super(
      options.materializedViewName,
      [
        createMaterializedView({
          name: options.materializedViewName,
          destinationTable: options.tableName,
          select: selectStatement,
        }),
        populateTable({
          destinationTable: options.tableName,
          select: selectStatement,
        }),
      ],
      [dropView(options.materializedViewName)],
      {
        pullsDataFrom: options.selectTables,
        pushesDataTo: [targetTable],
      },
    );

    this.targetTable = targetTable;
  }
}
