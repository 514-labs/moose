/**
 * OlapTable class for managing ClickHouse tables with typed interfaces
 */

import { Column } from "../dataModels/dataModelTypes";
import { IJsonSchemaCollection } from "typia/src/schemas/json/IJsonSchemaCollection";
import { getMooseInternal } from "./internal";
import { TypedBase, TypiaValidators } from "./typedBase";
import { Readable } from "stream";
import {
  OlapConfig,
  FailedRecord,
  InsertResult,
  InsertOptions,
  ValidationError,
  ValidationResult,
} from "./types";

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
