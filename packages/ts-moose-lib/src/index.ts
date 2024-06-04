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
