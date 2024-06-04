import ts from "typescript";

export type EnumValues =
  | { name: string; value: { Int: number } }[]
  | { name: string; value: { String: string } }[];
export type DataType =
  | string
  | { name: string; values: EnumValues }
  | { elementType: DataType };
export interface Column {
  name: string;
  dataType: DataType;
  required: boolean;
  unique: false; // what is this for?
  isKey: boolean;
  default: null;
}

export class UnknownType extends Error {
  t: ts.Type;
  constructor(t: ts.Type) {
    super();
    this.t = t;
  }
}

export class UnsupportedEnum extends Error {
  name: string;
  constructor(name: string) {
    super();
    this.name = name;
  }
}
