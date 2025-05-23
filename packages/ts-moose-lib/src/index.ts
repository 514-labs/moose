import { JWTPayload } from "jose";
import { MooseClient, sql } from "./consumption-apis/helpers";

export type Key<T extends string | number | Date> = T;

export type JWT<T extends object> = T;

export interface ConsumptionUtil {
  client: MooseClient;

  // SQL interpolator
  sql: typeof sql;
  jwt: JWTPayload | undefined;
}

export type DataModelConfig<T> = Partial<{
  ingestion: true;
  storage: {
    enabled?: boolean;
    order_by_fields?: (keyof T)[];
    deduplicate?: boolean;
    name?: string;
  };
  parallelism?: number;
}>;

export * from "./blocks/helpers";
export * from "./commons";
export * from "./consumption-apis/helpers";
export * from "./scripts/task";
export {
  OlapTable,
  Stream,
  DeadLetterModel,
  DeadLetterQueue,
  IngestApi,
  ConsumptionApi,
  IngestPipeline,
  SqlResource,
  View,
  MaterializedView,
} from "./dmv2";

export { createConsumptionApi } from "./consumption-apis/runner";

export {
  ClickHousePrecision,
  ClickHouseDecimal,
  ClickHouseInt,
  LowCardinality,
} from "./dataModels/types";
