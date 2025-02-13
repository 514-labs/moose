import ts from "typescript";

export type EnumValues =
  | { name: string; value: { Int: number } }[]
  | { name: string; value: { String: string } }[];
export type DataEnum = { name: string; values: EnumValues };
export type Nested = { name: string; columns: Column[]; jwt: boolean };
export type ArrayType = { elementType: DataType; elementNullable: boolean };
export type DataType = string | DataEnum | ArrayType | Nested;
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
  enumName: string;
  constructor(enumName: string) {
    super();
    this.enumName = enumName;
  }
}

export class UnsupportedFeature extends Error {
  featureName: string;
  constructor(featureName: string) {
    super();
    this.featureName = featureName;
  }
}
