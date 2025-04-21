import { Pattern, TagBase } from "typia/lib/tags";

export type ClickHousePrecision<P extends number> = {
  _clickhouse_precision?: P;
};

export const DecimalRegex: "^-?\\d+(\\.\\d+)?$" = "^-?\\d+(\\.\\d+)?$";

export type ClickHouseDecimal<P extends number, S extends number> = {
  _clickhouse_precision?: P;
  _clickhouse_scale?: S;
} & Pattern<typeof DecimalRegex>;

export type ClickHouseInt<
  Value extends "int8" | "int16",
  // | "int32"
  // | "int64"
  // | "int128"
  // | "int256"
  // | "uInt8"
  // | "uInt16"
  // | "uInt32"
  // | "uInt64"
  // | "uInt128"
  // | "uInt256",
> = TagBase<{
  target: "number";
  kind: "type";
  value: Value;
  validate: Value extends "int8"
    ? "-128 <= $input && $input <= 127"
    : "-32768 <= $input && $input <= 32767";
  exclusive: true;
  schema: {
    type: "integer";
  };
}>;
