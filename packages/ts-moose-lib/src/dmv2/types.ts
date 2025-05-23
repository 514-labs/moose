/**
 * Shared types and interfaces for the dmv2 module
 */

import { JWTPayload } from "jose";
import { MooseClient, sql } from "../consumption-apis/helpers";
import { DeadLetterQueue } from "./deadLetterQueue";


export type ZeroOrMany<T> = T | T[] | undefined | null;
export type SyncOrAsyncTransform<T, U> = (
  record: T,
) => ZeroOrMany<U> | Promise<ZeroOrMany<U>>;
export type Consumer<T> = (record: T) => Promise<void> | void;


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
  deadLetterQueue?: DeadLetterQueue<T>; // DeadLetterQueue<T> - circular dependency, typed as any
}

export interface ConsumerConfig<T> {
  version?: string;
  deadLetterQueue?:  DeadLetterQueue<T>; // DeadLetterQueue<T> - circular dependency, typed as any
}

export interface ConsumptionUtil {
  client: MooseClient;

  // SQL interpolator
  sql: typeof sql;
  jwt: JWTPayload | undefined;
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
