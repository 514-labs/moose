import * as typia from "typia";

export interface QueryFieldDefinition {
  name: string;
  type: "string" | "number" | "boolean" | "date" | "array";
  elementType?: QueryFieldDefinition; // For arrays
  required: boolean;
  typiaType?: string;
}

export interface QueryParamMetadata {
  fields: QueryFieldDefinition[];
  typiaValidator?: ReturnType<typeof typia.createIs<any>>;
}
