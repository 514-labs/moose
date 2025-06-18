import { DataSource, DataSourceConfig } from "./dataSource";
import { Readable } from "node:stream";

/**
 * Type guard to check if the extraction result is a stream
 */
export function isStreamResult(result: any): result is Readable {
  return result instanceof Readable;
}

/**
 * Type guard to check if the extraction result is a single response
 */
export function isSingleResult<T>(result: T | Readable): result is T {
  return !(result instanceof Readable);
}

/**
 * Configuration for pagination behavior
 */
export interface PaginationConfig<T = any> {
  /** Function to extract the next URL from the response */
  getNextUrl: (response: T, headers: Headers) => string | null;

  /** Maximum number of pages to fetch (safety limit) */
  maxPages?: number;

  /** Delay between requests in milliseconds (rate limiting) */
  delayBetweenRequests?: number;

  /** Retry configuration for failed requests */
  retryConfig?: {
    maxRetries: number;
    backoffMs: number;
  };
}

export interface APISourceConfig<T = any, ItemType = any>
  extends DataSourceConfig {
  baseUrl: string;
  endpoint: string;
  auth?: {
    token: string;
  };
  /** Custom headers to include in all requests */
  headers?: Record<string, string>;
  /** Function to extract the data items from the response */
  extractItems: (response: T, headers: Headers) => ItemType;
  pagination?: PaginationConfig<T>;
}

export class APISource<T = any, ItemType = any>
  extends DataSource<T, ItemType>
  implements
    AsyncIterable<ItemType extends readonly any[] ? ItemType[number] : ItemType>
{
  private baseUrl: string;
  private endpoint: string;
  private auth?: { token: string };
  private headers?: Record<string, string>;
  private extractItems: (response: T, headers: Headers) => ItemType;
  private pagination?: PaginationConfig<T>;

  constructor(config: APISourceConfig<T, ItemType>) {
    super(config);
    this.baseUrl = config.baseUrl;
    this.endpoint = config.endpoint;
    this.auth = config.auth;
    this.headers = config.headers;
    this.extractItems = config.extractItems;
    this.pagination = config.pagination;
  }

  /**
   * Extracts data from the API endpoint.
   *
   * For single requests (no pagination): returns the extracted data directly.
   * For paginated requests: returns a Readable stream that yields individual items.
   *
   * @returns Promise resolving to either the extracted data or a Readable stream
   */
  async extract(): Promise<ItemType | Readable> {
    // If no pagination config, use single request behavior
    if (!this.pagination) {
      return this.extractSingle();
    }

    // Create a readable stream for paginated results
    const stream = new Readable({
      objectMode: true,
      // paginateIntoStream will push data to stream
      read() {},
    });

    // Start pagination in background
    this.paginateIntoStream(stream).catch((error) => {
      stream.destroy(error);
    });

    return stream;
  }

  /**
   * Tests the connection to the API endpoint.
   *
   * Attempts a HEAD request first (efficient), falls back to GET if needed.
   * Includes authentication and custom headers in the test request.
   *
   * @returns Promise resolving to connection test result with success status and message
   */
  async testConnection(): Promise<{ success: boolean; message?: string }> {
    try {
      const url = `${this.baseUrl}${this.endpoint}`;
      const headers: Record<string, string> = {
        "Content-Type": "application/json",
      };

      // Add custom headers if provided
      if (this.headers) {
        Object.assign(headers, this.headers);
      }

      if (this.auth?.token) {
        headers["Authorization"] = `Bearer ${this.auth.token}`;
      }

      // Try HEAD request first (most efficient - no body transfer)
      const response = await fetch(url, {
        method: "HEAD",
        headers,
      });

      if (response.ok) {
        return {
          success: true,
          message: `API connection successful (${response.status})`,
        };
      }

      // If HEAD fails, try GET as fallback
      const getResponse = await fetch(url, {
        method: "GET",
        headers,
      });

      if (getResponse.ok) {
        return {
          success: true,
          message: `API connection successful via GET (${getResponse.status})`,
        };
      }

      return {
        success: false,
        message: `API returned error: ${getResponse.status} ${getResponse.statusText}`,
      };
    } catch (error) {
      return {
        success: false,
        message: `Connection failed: ${error instanceof Error ? error.message : "Unknown error"}`,
      };
    }
  }

  /**
   * Make a single HTTP request with retry logic
   */
  private async makeRequest(
    url: string,
    retryCount: number = 0,
  ): Promise<{ data: T; headers: Headers }> {
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    };

    // Add custom headers if provided
    if (this.headers) {
      Object.assign(headers, this.headers);
    }

    if (this.auth?.token) {
      headers["Authorization"] = `Bearer ${this.auth.token}`;
    }

    try {
      const response = await fetch(url, {
        method: "GET",
        headers,
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();

      // Return the actual Headers object for better API
      return { data, headers: response.headers };
    } catch (error) {
      const maxRetries = this.pagination?.retryConfig?.maxRetries ?? 0;
      if (retryCount < maxRetries) {
        const backoffMs = this.pagination?.retryConfig?.backoffMs ?? 1000;
        await new Promise((resolve) =>
          setTimeout(resolve, backoffMs * (retryCount + 1)),
        );
        return this.makeRequest(url, retryCount + 1);
      }
      throw error;
    }
  }

  /**
   * Extract data from a single API request (non-paginated)
   */
  private async extractSingle(): Promise<ItemType> {
    try {
      const url = `${this.baseUrl}${this.endpoint}`;
      const { data, headers } = await this.makeRequest(url);
      return this.extractItems(data, headers);
    } catch (error) {
      throw new Error(
        `Extraction failed: ${error instanceof Error ? error.message : "Unknown error"}`,
      );
    }
  }

  /**
   * Paginate through API responses and push items to stream
   */
  private async paginateIntoStream(stream: Readable): Promise<void> {
    const {
      getNextUrl,
      maxPages = 100,
      delayBetweenRequests = 0,
    } = this.pagination!;
    let currentUrl = `${this.baseUrl}${this.endpoint}`;
    let pageCount = 1;

    try {
      while (currentUrl && pageCount <= maxPages) {
        console.log(`Fetching page ${pageCount} from ${currentUrl}`);
        const { data: response, headers } = await this.makeRequest(currentUrl);
        const items = this.extractItems(response, headers);

        // Handle different types of items
        if (Array.isArray(items)) {
          // If it's an array, push each item
          for (const item of items) {
            stream.push(item);
          }
        } else {
          // If it's a single item, push it directly
          stream.push(items);
        }

        // Get next URL
        const nextUrl = getNextUrl(response, headers);
        currentUrl = nextUrl || "";
        pageCount++;

        // Rate limiting
        if (delayBetweenRequests > 0) {
          await new Promise((resolve) =>
            setTimeout(resolve, delayBetweenRequests),
          );
        }
      }

      // End stream
      stream.push(null);
    } catch (error) {
      stream.destroy(error instanceof Error ? error : new Error(String(error)));
    }
  }

  /**
   * Makes the APISource iterable for easy data consumption.
   *
   * How it works:
   * 1. When you use `for await (const item of apisource)`, JavaScript calls this method
   * 2. This method calls `extract()` to get the data (stream or single response)
   * 3. If it's a stream (paginated data), it yields each item from the stream
   * 4. If it's a single response, it yields the data directly
   * 5. The `yield` keyword pauses execution and gives the item to the caller
   * 6. When the caller asks for the next item, execution continues
   *
   */
  async *[Symbol.asyncIterator](): AsyncIterator<
    ItemType extends readonly any[] ? ItemType[number] : ItemType
  > {
    const result = await this.extract();

    if (isStreamResult(result)) {
      // For paginated results, yield items from the stream
      for await (const item of result) {
        yield item as ItemType extends readonly any[] ? ItemType[number]
        : ItemType;
      }
    } else {
      // For single results, yield the data directly
      yield result as ItemType extends readonly any[] ? ItemType[number]
      : ItemType;
    }
  }
}
