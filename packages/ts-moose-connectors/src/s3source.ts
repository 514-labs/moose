import {
  GetObjectCommand,
  ListObjectsV2Command,
  HeadBucketCommand,
  S3Client,
} from "@aws-sdk/client-s3";
import { Readable } from "node:stream";
import {
  parseCSV,
  parseJSON,
  DEFAULT_CSV_CONFIG,
  DEFAULT_JSON_CONFIG,
} from "./utilities/dataParser";

/**
 * Configuration for S3 data source.
 * Uses unions to enforce proper configuration based on format.
 */
export interface S3ConfigBase {
  /** S3 bucket name */
  bucket: string;
  /** S3 key prefix to filter objects */
  prefix: string;
  /** Authentication credentials for AWS S3 */
  auth: {
    accessKeyId: string;
    accessKeySecret: string;
    region: string;
    // Optional, user may need it for temporary credentials
    sessionToken?: string;
  };
}

/**
 * Configuration for JSON format S3 data source.
 */
export interface S3ConfigJSON extends S3ConfigBase {
  format: "json";
}

/**
 * Configuration for CSV format S3 data source.
 * Requires CSV-specific parsing options.
 */
export interface S3ConfigCSV extends S3ConfigBase {
  format: "csv";
  csv: {
    delimiter: string;
  };
}

/**
 * Union type for S3 configuration. Enforces proper configuration based on format.
 */
export type S3Config = S3ConfigJSON | S3ConfigCSV;

/**
 * S3 data source that can read and parse files from an S3 bucket.
 *
 * @template T - The TypeScript interface type for parsed data items
 */
export class S3Source<T> implements AsyncIterable<T> {
  private config: S3Config;
  private s3: S3Client;

  constructor(config: S3Config) {
    this.config = config;
    this.s3 = new S3Client({
      region: this.config.auth.region,
      credentials: {
        accessKeyId: this.config.auth.accessKeyId,
        secretAccessKey: this.config.auth.accessKeySecret,
        sessionToken: this.config.auth.sessionToken,
      },
    });
  }

  /**
   * Tests the connection to the S3 bucket.
   * Performs a HEAD request to verify bucket access and permissions.
   *
   * @returns Promise resolving to connection test result with success status and message
   */
  public async testConnection(): Promise<{
    success: boolean;
    message?: string;
  }> {
    try {
      const response = await this.s3.send(
        new HeadBucketCommand({
          Bucket: this.config.bucket,
        }),
      );

      return { success: true, message: `S3 connection successful` };
    } catch (error) {
      return {
        success: false,
        message: `S3 connection failed: ${error instanceof Error ? error.message : "Unknown error"}`,
      };
    }
  }

  /**
   * Extracts data from S3 files and returns a readable stream.
   *
   * @returns Promise resolving to a Readable stream of parsed data items
   */
  public async extract(): Promise<Readable> {
    try {
      const files = await this.listFiles();

      const stream = new Readable({
        objectMode: true,
        read() {},
      });

      this.processFilesToStream(files, stream).catch((error) => {
        stream.destroy(error);
      });

      return stream;
    } catch (error) {
      console.error(
        `S3 extraction failed: ${error instanceof Error ? error.message : "Unknown error"}`,
      );
      throw error;
    }
  }

  /**
   * Lists all files in the S3 bucket that match the configured prefix and format.
   * Handles pagination automatically for large buckets.
   */
  private async listFiles(): Promise<string[]> {
    const files: string[] = [];

    try {
      let continuationToken: string | undefined;

      do {
        const listCommand = new ListObjectsV2Command({
          Bucket: this.config.bucket,
          Prefix: this.config.prefix,
          ContinuationToken: continuationToken,
        });

        const response = await this.s3.send(listCommand);

        if (response.Contents) {
          for (const object of response.Contents) {
            if (object.Key && object.Key.endsWith(`.${this.config.format}`)) {
              files.push(object.Key);
            }
          }
        }

        continuationToken = response.NextContinuationToken;
      } while (continuationToken);

      console.log(`Found ${files.length} files in S3 bucket`);
      return files;
    } catch (error) {
      console.error(
        `S3 list files failed: ${error instanceof Error ? error.message : "Unknown error"}`,
      );
      throw error;
    }
  }

  /**
   * Processes files and pushes parsed data items to the provided stream.
   */
  private async processFilesToStream(
    files: string[],
    stream: Readable,
  ): Promise<void> {
    try {
      const totalFiles = files.length;
      let processedFile = 1;

      for (const file of files) {
        console.log(
          `Processing S3 file: ${file} (${processedFile++}/${totalFiles})`,
        );

        const content = await this.getFileContent(file);

        if (this.config.format === "json") {
          try {
            const parsedData = parseJSON<T>(content, DEFAULT_JSON_CONFIG);

            for (const item of parsedData) {
              stream.push(item);
            }
          } catch (parseError) {
            console.error(
              `Failed to parse JSON from ${file}: ${parseError instanceof Error ? parseError.message : "Unknown error"}`,
            );
          }
        } else if (this.config.format === "csv") {
          try {
            const csvConfig = this.config as S3ConfigCSV;
            const csvData = await parseCSV<T>(content, {
              ...DEFAULT_CSV_CONFIG,
              delimiter: csvConfig.csv.delimiter,
            });

            for (const item of csvData) {
              stream.push(item);
            }
          } catch (parseError) {
            console.error(
              `Failed to parse CSV from ${file}: ${parseError instanceof Error ? parseError.message : "Unknown error"}`,
            );
          }
        }
      }

      stream.push(null);
    } catch (error) {
      stream.destroy(error instanceof Error ? error : new Error(String(error)));
    }
  }

  /**
   * Retrieves the content of a single file from S3.
   */
  private async getFileContent(file: string): Promise<string> {
    try {
      const getCommand = new GetObjectCommand({
        Bucket: this.config.bucket,
        Key: file,
      });

      const response = await this.s3.send(getCommand);

      if (!response.Body) {
        throw new Error(`No content found for file: ${file}`);
      }

      return await response.Body.transformToString();
    } catch (error) {
      console.error(
        `Failed to get content for file ${file}: ${error instanceof Error ? error.message : "Unknown error"}`,
      );
      throw error;
    }
  }

  /**
   * Makes the S3Source iterable for easy data consumption.
   *
   * How it works:
   * 1. When you use `for await (const item of s3source)`, JavaScript calls this method
   * 2. This method calls `extract()` to get the data stream
   * 3. It yields each item from the stream
   * 4. The `yield` keyword pauses execution and gives the item to the caller
   * 5. When the caller asks for the next item, execution continues
   *
   */
  async *[Symbol.asyncIterator](): AsyncIterator<T> {
    const result = await this.extract();

    for await (const item of result) {
      yield item as T;
    }
  }
}
