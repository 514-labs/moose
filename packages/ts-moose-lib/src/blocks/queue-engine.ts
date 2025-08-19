/**
 * Queue Engine Types for Moose
 * 
 * Provides backend-agnostic queue table engine support for streaming data ingestion.
 * Currently supports S3Queue and AzureQueue with comprehensive configuration options.
 */

export interface QueueEngine {
  source: QueueSource;
  processing?: ProcessingConfig;
  coordination?: CoordinationConfig;
  monitoring?: MonitoringConfig;
}

export type QueueSource = S3QueueSource | AzureQueueSource;

export interface S3QueueSource {
  type: 's3';
  path: string;
  format: string;
  credentials?: S3Credentials;
  /** Additional S3-specific settings */
  extraSettings?: Record<string, string>;
}

export interface AzureQueueSource {
  type: 'azure';
  container: string;
  path: string;
  format: string;
  /** Additional Azure-specific settings */
  extraSettings?: Record<string, string>;
}

export interface S3Credentials {
  roleArn?: string;
  accessKeyId?: string;
  secretAccessKey?: string;
  sessionToken?: string;
}

export interface ProcessingConfig {
  /** Processing order mode */
  mode?: ProcessingMode;
  /** What to do with files after successful processing */
  afterProcessing?: AfterProcessing;
  /** Number of retry attempts for failed files */
  retries?: number;
  /** Number of processing threads */
  threads?: number;
  /** Enable parallel inserts for better throughput */
  parallelInserts?: boolean;
  /** Number of logical processing units for distributed processing */
  buckets?: number;
}

export type ProcessingMode = 'ordered' | 'unordered';
export type AfterProcessing = 'keep' | 'delete';

export interface CoordinationConfig {
  /** Custom coordination path */
  path?: string;
  /** Maximum number of tracked files for unordered mode */
  trackedFilesLimit?: number;
  /** TTL in seconds for tracked files */
  trackedFileTtlSec?: number;
  /** Minimum cleanup interval in milliseconds */
  cleanupIntervalMinMs?: number;
  /** Maximum cleanup interval in milliseconds */
  cleanupIntervalMaxMs?: number;
}

export interface MonitoringConfig {
  /** Enable logging to system tables */
  enableLogging?: boolean;
  /** Minimum polling timeout in milliseconds */
  pollingMinTimeoutMs?: number;
  /** Maximum polling timeout in milliseconds */
  pollingMaxTimeoutMs?: number;
  /** Additional backoff time when no files found */
  pollingBackoffMs?: number;
}

/**
 * Create an S3Queue engine configuration with sensible defaults
 */
export function createS3QueueEngine(
  path: string,
  format: string,
  options?: Partial<Omit<QueueEngine, 'source'>>
): QueueEngine {
  return {
    source: {
      type: 's3',
      path,
      format,
    },
    processing: {
      mode: 'unordered',
      afterProcessing: 'keep',
      retries: 0,
      parallelInserts: false,
      ...options?.processing,
    },
    coordination: {
      trackedFilesLimit: 1000,
      cleanupIntervalMinMs: 10000,
      cleanupIntervalMaxMs: 30000,
      ...options?.coordination,
    },
    monitoring: {
      enableLogging: false,
      pollingMinTimeoutMs: 1000,
      pollingMaxTimeoutMs: 10000,
      pollingBackoffMs: 0,
      ...options?.monitoring,
    },
  };
}

/**
 * Create an AzureQueue engine configuration with sensible defaults
 */
export function createAzureQueueEngine(
  container: string,
  path: string,
  format: string,
  options?: Partial<Omit<QueueEngine, 'source'>>
): QueueEngine {
  return {
    source: {
      type: 'azure',
      container,
      path,
      format,
    },
    processing: {
      mode: 'unordered',
      afterProcessing: 'keep',
      retries: 0,
      parallelInserts: false,
      ...options?.processing,
    },
    coordination: {
      trackedFilesLimit: 1000,
      cleanupIntervalMinMs: 10000,
      cleanupIntervalMaxMs: 30000,
      ...options?.coordination,
    },
    monitoring: {
      enableLogging: false,
      pollingMinTimeoutMs: 1000,
      pollingMaxTimeoutMs: 10000,
      pollingBackoffMs: 0,
      ...options?.monitoring,
    },
  };
}

/**
 * Validate queue engine configuration
 */
export function validateQueueEngine(engine: QueueEngine): string[] {
  const errors: string[] = [];

  // Validate source
  if (engine.source.type === 's3') {
    const s3Source = engine.source as S3QueueSource;
    if (!s3Source.path) {
      errors.push('S3 path is required');
    }
    if (!s3Source.format) {
      errors.push('S3 format is required');
    }
    if (s3Source.path && !s3Source.path.startsWith('s3://')) {
      errors.push('S3 path must start with "s3://"');
    }
  } else if (engine.source.type === 'azure') {
    const azureSource = engine.source as AzureQueueSource;
    if (!azureSource.container) {
      errors.push('Azure container is required');
    }
    if (!azureSource.path) {
      errors.push('Azure path is required');
    }
    if (!azureSource.format) {
      errors.push('Azure format is required');
    }
  }

  // Validate processing config
  if (engine.processing?.buckets !== undefined && engine.processing.buckets <= 0) {
    errors.push('Buckets must be greater than 0');
  }

  if (engine.processing?.retries !== undefined && engine.processing.retries < 0) {
    errors.push('Retries must be non-negative');
  }

  if (engine.processing?.threads !== undefined && engine.processing.threads <= 0) {
    errors.push('Threads must be greater than 0');
  }

  // Validate ordered mode constraints
  if (engine.processing?.mode === 'ordered') {
    if (engine.processing.buckets && engine.processing.threads) {
      if (engine.processing.buckets < engine.processing.threads) {
        errors.push('For ordered mode, buckets should be >= threads');
      }
    }
  }

  return errors;
}

/**
 * Common ClickHouse formats supported by queue engines
 */
export const SUPPORTED_FORMATS = [
  'JSONEachRow',
  'CSV',
  'TSV',
  'Parquet',
  'JSONStringsEachRow',
  'JSONCompactEachRow',
  'CSVWithNames',
  'TSVWithNames',
  'TabSeparated',
  'TabSeparatedWithNames',
  'TSKV',
  'Avro',
  'ORC',
] as const;

export type SupportedFormat = typeof SUPPORTED_FORMATS[number];

/**
 * Example usage configurations for common scenarios
 */
export const EXAMPLE_CONFIGS = {
  /**
   * Basic S3Queue for JSON logs
   */
  s3JsonLogs: (bucket: string, prefix: string): QueueEngine => 
    createS3QueueEngine(`s3://${bucket}/${prefix}/*.json`, 'JSONEachRow'),

  /**
   * High-throughput S3Queue with parallel processing
   */
  s3HighThroughput: (bucket: string, prefix: string): QueueEngine =>
    createS3QueueEngine(`s3://${bucket}/${prefix}/*.json`, 'JSONEachRow', {
      processing: {
        mode: 'unordered',
        threads: 8,
        parallelInserts: true,
        buckets: 16,
        retries: 3,
      },
      monitoring: {
        enableLogging: true,
        pollingMinTimeoutMs: 500,
        pollingMaxTimeoutMs: 5000,
      },
    }),

  /**
   * Ordered processing for sequential data
   */
  s3OrderedProcessing: (bucket: string, prefix: string): QueueEngine =>
    createS3QueueEngine(`s3://${bucket}/${prefix}/*.csv`, 'CSV', {
      processing: {
        mode: 'ordered',
        buckets: 4,
        threads: 2,
        retries: 5,
      },
      coordination: {
        trackedFileTtlSec: 3600, // 1 hour
        trackedFilesLimit: 5000,
      },
    }),

  /**
   * Development/testing configuration
   */
  s3Development: (bucket: string, prefix: string): QueueEngine =>
    createS3QueueEngine(`s3://${bucket}/${prefix}/*.json`, 'JSONEachRow', {
      processing: {
        retries: 1,
      },
      monitoring: {
        enableLogging: true,
        pollingMinTimeoutMs: 2000,
        pollingMaxTimeoutMs: 10000,
      },
    }),
} as const;