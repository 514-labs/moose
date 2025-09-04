import { quoteIdentifier } from "../sqlHelpers";

interface AggregationCreateOptions {
  tableCreateOptions: TableCreateOptions;
  materializedViewName: string;
  select: string;
}

interface AggregationDropOptions {
  viewName: string;
  tableName: string;
}

interface MaterializedViewCreateOptions {
  name: string;
  destinationTable: string;
  select: string;
}

interface PopulateTableOptions {
  destinationTable: string;
  select: string;
}

interface TableCreateOptions {
  name: string;
  columns: Record<string, string>;
  engine?: ClickHouseEngines;
  orderBy?: string;
}

export interface Blocks {
  setup: string[];
  teardown: string[];
}

export enum ClickHouseEngines {
  MergeTree = "MergeTree",
  ReplacingMergeTree = "ReplacingMergeTree",
  SummingMergeTree = "SummingMergeTree",
  AggregatingMergeTree = "AggregatingMergeTree",
  CollapsingMergeTree = "CollapsingMergeTree",
  VersionedCollapsingMergeTree = "VersionedCollapsingMergeTree",
  GraphiteMergeTree = "GraphiteMergeTree",
  S3Queue = "S3Queue",
}

/**
 * Drops an existing view if it exists.
 */
export function dropView(name: string): string {
  return `DROP VIEW IF EXISTS ${quoteIdentifier(name)}`.trim();
}

/**
 * Creates a materialized view.
 */
export function createMaterializedView(
  options: MaterializedViewCreateOptions,
): string {
  return `CREATE MATERIALIZED VIEW IF NOT EXISTS ${quoteIdentifier(options.name)}
        TO ${quoteIdentifier(options.destinationTable)}
        AS ${options.select}`.trim();
}

/**
 * Populates a table with data.
 */
export function populateTable(options: PopulateTableOptions): string {
  return `INSERT INTO ${quoteIdentifier(options.destinationTable)}
          ${options.select}`.trim();
}
