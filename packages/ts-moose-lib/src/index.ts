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
    deduplicate?: boolean;
  };
  parallelism?: number;
}>;

export * from "./blocks/helpers";
export * from "./commons";
export * from "./consumption-apis/helpers";

export { createConsumptionApi } from "./consumption-apis/runner";
export type { ConsumptionApiConfig } from "./consumption-apis/runner";
export type {
  QueryFieldDefinition,
  QueryParamMetadata,
} from "./consumption-apis/types";
