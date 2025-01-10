export interface QueryFieldDefinition {
  name: string;
  type: "string" | "number" | "boolean" | "date" | "array";
  elementType?: QueryFieldDefinition; // For arrays
  required: boolean;
}

export interface QueryParamMetadata {
  fields: QueryFieldDefinition[];
}
