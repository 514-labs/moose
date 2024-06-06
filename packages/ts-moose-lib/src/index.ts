import { MooseClient, sql } from "./consumption-helpers";
export interface Aggregation {
  select: string;
  orderBy: string;
}

export type Key<T extends string | number | Date> = T;

export interface ConsumptionUtil {
  client: MooseClient;

  // SQL interpolator
  sql: typeof sql;
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

export * from "./commons";
export * from "./consumption-helpers";
