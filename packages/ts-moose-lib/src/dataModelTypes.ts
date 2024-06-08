import ts from "typescript";

export type EnumValues =
  | { name: string; value: { Int: number } }[]
  | { name: string; value: { String: string } }[];
export type DataEnum = { name: string; values: EnumValues };
export type Nested = { name: string; columns: Column[] };
export type DataType = string | DataEnum | { elementType: DataType } | Nested;
export interface Column {
  name: string;
  data_type: DataType;
  required: boolean;
  unique: false; // what is this for?
  primary_key: boolean;
  default: null;
}

export interface DataModel {
  columns: Column[];
  name: string;
}

export class UnknownType extends Error {
  t: ts.Type;
  fieldName: string;
  typeName: string;
  constructor(t: ts.Type, fieldName: string, typeName: string) {
    super();
    this.t = t;
    this.fieldName = fieldName;
    this.typeName = typeName;
  }
}

export class UnsupportedEnum extends Error {
  enum_name: string;
  constructor(enum_name: string) {
    super();
    this.enum_name = enum_name;
  }
}
