export type Key<T extends string | number | Date> = T;

export type JWT<T extends object> = T;

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
  Aggregated,
  OlapTable,
  OlapConfig,
  Stream,
  StreamConfig,
  DeadLetterModel,
  DeadLetter,
  DeadLetterQueue,
  IngestApi,
  IngestConfig,
  ConsumptionApi,
  EgressConfig,
  IngestPipeline,
  SqlResource,
  View,
  MaterializedView,
  Task,
  Workflow,
} from "./dmv2";

export { createConsumptionApi } from "./consumption-apis/runner";

export {
  ClickHousePrecision,
  ClickHouseDecimal,
  ClickHouseByteSize,
  ClickHouseInt,
  LowCardinality,
  ClickHouseNamedTuple,
} from "./dataModels/types";

export { MooseCache } from "./clients/redisClient";

export { ConsumptionUtil } from "./consumption-apis/helpers";

export * from "./connectors/apiSource";
