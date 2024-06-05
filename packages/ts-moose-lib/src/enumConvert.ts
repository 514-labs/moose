import ts, {
  NumberLiteralType,
  StringLiteralType,
  UnionType,
} from "typescript";
import { DataEnum, DataType, UnsupportedEnum } from "./dataModelTypes";

export const isEnum = (t: ts.Type): boolean =>
  !!(t.getFlags() & ts.TypeFlags.EnumLiteral);

export const enumConvert = (enumType: ts.Type): DataEnum => {
  const name = enumType.symbol.name;

  // an enum is the union of the values
  const values = (enumType as UnionType).types;
  const allStrings = values.every((v) => v.isStringLiteral());
  const allIntegers = values.every(
    (v) => v.isNumberLiteral() && Number.isInteger(v.value),
  );

  if (!allIntegers && !allStrings) {
    throw new UnsupportedEnum(name);
  }

  const enumMember = allStrings
    ? values.map((v) => ({
        name: v.symbol.name,
        value: { String: (v as StringLiteralType).value },
      }))
    : values.map((v) => ({
        name: v.symbol.name,
        value: { Int: (v as NumberLiteralType).value },
      }));

  return { name, values: enumMember };
};
