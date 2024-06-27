interface AggregationBlock {
  name: string;
  select: string;
  orderBy: string;
}

export interface Blocks {
  setup: string[];
  teardown: string[];
}

export function createAggregation(aggregation: AggregationBlock): string {
  return `CREATE MATERIALIZED VIEW IF NOT EXISTS ${aggregation.name} 
        ENGINE = AggregatingMergeTree() ORDER BY ${aggregation.orderBy} 
        POPULATE 
        AS ${aggregation.select}`;
}

export function dropAggregation(name: string): string {
  return `DROP VIEW IF EXISTS ${name}`;
}
