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

export type ClickHouseByteSize<N extends number> = {
  _clickhouse_byte_size?: N;
};

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
> =
  Value extends "int32" | "int64" | "uint32" | "uint64" ? tags.Type<Value>
  : TagBase<{
      target: "number";
      kind: "type";
      value: Value;
      validate: Value extends "int8" ? "-128 <= $input && $input <= 127"
      : Value extends "int16" ? "-32768 <= $input && $input <= 32767"
      : Value extends "uint8" ? "0 <= $input && $input <= 255"
      : Value extends "uint16" ? "0 <= $input && $input <= 65535"
      : never;
      exclusive: true;
      schema: {
        type: "integer";
      };
    }>;

/**
 * By default, nested objects map to the `Nested` type in clickhouse.
 * Write `nestedObject: AnotherInterfaceType & ClickHouseNamedTuple`
 * to map AnotherInterfaceType to the named tuple type.
 */
export type ClickHouseNamedTuple = {
  _clickhouse_mapped_type?: "namedTuple";
};

/**
 * Geo type definitions for ClickHouse spatial data types.
 * These types represent geospatial data in Well-Known Text (WKT) format.
 */

/** Point represents a single coordinate pair (x, y) */
export type Point = string & { readonly __brand: 'Point' };

/** Ring represents a closed line string forming a simple polygon boundary */
export type Ring = string & { readonly __brand: 'Ring' };

/** Polygon represents a closed area defined by one or more rings */
export type Polygon = string & { readonly __brand: 'Polygon' };

/** MultiPolygon represents multiple polygons as a single geometry */
export type MultiPolygon = string & { readonly __brand: 'MultiPolygon' };

/** LineString represents a sequence of connected points forming a line */
export type LineString = string & { readonly __brand: 'LineString' };

/** MultiLineString represents multiple line strings as a single geometry */
export type MultiLineString = string & { readonly __brand: 'MultiLineString' };
