import { parse } from "csv-parse";
import { jsonDateReviver } from "./json";

/**
 * Configuration for CSV parsing options
 */
export interface CSVParsingConfig {
  /** CSV delimiter character */
  delimiter: string;
  /** Whether to treat first row as headers */
  columns?: boolean;
  /** Whether to skip empty lines */
  skipEmptyLines?: boolean;
  /** Whether to trim whitespace from values */
  trim?: boolean;
}

/**
 * Configuration for JSON parsing options
 */
export interface JSONParsingConfig {
  /** Custom reviver function for JSON.parse */
  reviver?: (key: string, value: any) => any;
}

/**
 * Parses CSV content into an array of objects
 *
 * @param content - The CSV content as a string
 * @param config - CSV parsing configuration
 * @returns Promise resolving to an array of parsed objects
 */
export function parseCSV<T = Record<string, any>>(
  content: string,
  config: CSVParsingConfig,
): Promise<T[]> {
  return new Promise((resolve, reject) => {
    const results: T[] = [];

    parse(content, {
      delimiter: config.delimiter,
      columns: config.columns ?? true,
      skip_empty_lines: config.skipEmptyLines ?? true,
      trim: config.trim ?? true,
    })
      .on("data", (row) => {
        results.push(row as T);
      })
      .on("end", () => {
        resolve(results);
      })
      .on("error", (error) => {
        reject(error);
      });
  });
}

/**
 * Parses JSON content into an array of objects
 *
 * @param content - The JSON content as a string
 * @param config - JSON parsing configuration
 * @returns Array of parsed objects
 */
export function parseJSON<T = any>(
  content: string,
  config: JSONParsingConfig = {},
): T[] {
  try {
    const parsed = JSON.parse(content, config.reviver);

    // Handle both array and single object cases
    if (Array.isArray(parsed)) {
      return parsed as T[];
    } else {
      return [parsed as T];
    }
  } catch (error) {
    throw new Error(
      `Failed to parse JSON: ${error instanceof Error ? error.message : "Unknown error"}`,
    );
  }
}

/**
 * Parses JSON content with automatic date revival
 *
 * @param content - The JSON content as a string
 * @returns Array of parsed objects with Date objects for ISO 8601 strings
 */
export function parseJSONWithDates<T = any>(content: string): T[] {
  return parseJSON<T>(content, { reviver: jsonDateReviver });
}

/**
 * Type guard to check if a value is a valid CSV delimiter
 */
export function isValidCSVDelimiter(delimiter: string): boolean {
  return delimiter.length === 1 && !/\s/.test(delimiter);
}

/**
 * Common CSV delimiters
 */
export const CSV_DELIMITERS = {
  COMMA: ",",
  TAB: "\t",
  SEMICOLON: ";",
  PIPE: "|",
} as const;

/**
 * Default CSV parsing configuration
 */
export const DEFAULT_CSV_CONFIG: CSVParsingConfig = {
  delimiter: CSV_DELIMITERS.COMMA,
  columns: true,
  skipEmptyLines: true,
  trim: true,
};

/**
 * Default JSON parsing configuration with date revival
 */
export const DEFAULT_JSON_CONFIG: JSONParsingConfig = {
  reviver: jsonDateReviver,
};
