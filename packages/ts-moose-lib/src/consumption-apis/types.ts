import * as typia from "typia";

export interface QueryFieldDefinition {
  name: string;
  type: "string" | "number" | "boolean" | "date" | "array";
  elementType?: QueryFieldDefinition; // For arrays
  required: boolean;
}

export interface QueryParamMetadata {
  fields: QueryFieldDefinition[];
  typiaValidator?: ReturnType<typeof typia.createIs<any>>;
}

export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: any;
  };
}
