interface AggregationCreateOptions {
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
}

/**
 * Drops an existing aggregation if it exists.
 */
export function dropAggregation(name: string): string {
  return `DROP VIEW IF EXISTS ${name}`.trim();
}

/**
 * Drops an existing table if it exists.
 */
export function dropTable(name: string): string {
  return `DROP TABLE IF EXISTS ${name}`.trim();
}

/**
 * Creates a materialized view for aggregation purposes.
 */
export function createAggregation(options: AggregationCreateOptions): string {
  return `CREATE MATERIALIZED VIEW IF NOT EXISTS ${options.name} 
        TO ${options.destinationTable}
        AS ${options.select}`.trim();
}

/**
 * Creates a new table.
 */
export function createTable(options: TableCreateOptions): string {
  const columnDefinitions = Object.entries(options.columns)
    .map(([name, type]) => `${name} ${type}`)
    .join(",\n");

  const orderByClause = options.orderBy ? `ORDER BY ${options.orderBy}` : "";

  const engine = options.engine || ClickHouseEngines.MergeTree;

  return `
    CREATE TABLE IF NOT EXISTS ${options.name} 
    (
      ${columnDefinitions}
    )
    ENGINE = ${engine}()
    ${orderByClause}
  `.trim();
}

/**
 * Populates a table with data.
 */
export function populateTable(options: PopulateTableOptions): string {
  return `INSERT INTO ${options.destinationTable}
          ${options.select}`.trim();
}
