import { Readable } from "node:stream";

/**
 * Configuration for a data source
 */
export interface DataSourceConfig {
  name: string;
  supportsIncremental?: boolean;
}

/**
 * DataSource is an abstract class that defines the interface for all data sources.
 * It is used to extract data from a source and test the connection to the source.
 */
export abstract class DataSource<T = any, ItemType = any> {
  protected name: string;
  protected supportsIncremental: boolean;

  constructor(config: DataSourceConfig) {
    this.name = config.name;
    this.supportsIncremental = config.supportsIncremental ?? false;
  }

  /**
   * Extract data from the source
   * Returns either ItemType (for single requests) or Readable (for paginated requests)
   */
  abstract extract(): Promise<ItemType | Readable>;

  /**
   * Test connection to the source
   */
  abstract testConnection(): Promise<{ success: boolean; message?: string }>;
}

/**
 * Result returned from extraction
 * For single requests: data is of type T
 * For paginated requests: data is a Readable stream yielding items of type T
 */
export interface ExtractionResult<T = any> {
  data: T | Readable;
  metadata: Record<string, any>;
}
