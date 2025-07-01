export type Key<T extends string | number | Date> = T;

export type JWT<T extends object> = T;

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

export {
  ClickHousePrecision,
  ClickHouseDecimal,
  ClickHouseByteSize,
  ClickHouseInt,
  LowCardinality,
  ClickHouseNamedTuple,
} from "./dataModels/types";

export type { ConsumptionUtil } from "./consumption-apis/helpers";

export * from "./sqlHelpers";
