import { Pattern, TagBase } from "typia/lib/tags";
import { tags } from "typia";

export type ClickHousePrecision<P extends number> = {
  _clickhouse_precision?: P;
};

export const DecimalRegex: "^-?\\d+(\\.\\d+)?$" = "^-?\\d+(\\.\\d+)?$";

export type ClickHouseDecimal<P extends number, S extends number> = {
  _clickhouse_precision?: P;
  _clickhouse_scale?: S;
} & Pattern<typeof DecimalRegex>;

export type LowCardinality = {
  _LowCardinality?: true;
};

export type ClickHouseInt<
  Value extends
    | "int8"
    | "int16"
    | "int32"
    | "int64"
    // | "int128"
    // | "int256"
    | "uint8"
    | "uint16"
    | "uint32"
    | "uint64",
  // | "uint128"
  // | "uint256",
> = Value extends "int32" | "int64" | "uint32" | "uint64"
  ? tags.Type<Value>
  : TagBase<{
      target: "number";
      kind: "type";
      value: Value;
      validate: Value extends "int8"
        ? "-128 <= $input && $input <= 127"
        : Value extends "int16"
          ? "-32768 <= $input && $input <= 32767"
          : Value extends "uint8"
            ? "0 <= $input && $input <= 255"
            : Value extends "uint16"
              ? "0 <= $input && $input <= 65535"
              : never;
      exclusive: true;
      schema: {
        type: "integer";
      };
    }>;
