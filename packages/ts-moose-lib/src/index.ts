export interface Aggregation {
  select: string;
  orderBy: string;
}

export type Key<T extends string | number | Date> = T;

// the internal SQL type is currently not exposed to users
export type SQL = { readonly _tag: "SQL" };

// we can use this when we drop Deno in consumption
export interface ConsumptionUtil {
  client: {
    query: (sql: SQL) => Promise<any>;
  };

  // SQL interpolator
  sql: (strings: readonly string[], ...values: readonly (any | SQL)[]) => SQL;
}

export enum IngestionFormat {
  JSON = "JSON",
  JSON_ARRAY = "JSON_ARRAY",
}

export type DataModelConfig<T> = Partial<{
  ingestion: {
    format?: IngestionFormat;
  };
  storage: {
    enabled?: boolean;
    order_by_fields?: (keyof T)[];
  };
}>;
