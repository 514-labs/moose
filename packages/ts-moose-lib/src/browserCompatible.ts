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

export type { ConsumptionUtil } from "./consumption-apis/helpers";
