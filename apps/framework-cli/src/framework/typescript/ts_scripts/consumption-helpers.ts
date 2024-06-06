const typeMapping = {
  Int: "number",
  UInt: "number",
  UInt8: "number",
  LowCardinality: "string",
  UInt16: "number",
  UInt32: "number",
  UInt64: "string",
  UInt128: "string",
  UInt256: "string",
  Int8: "number",
  Int16: "number",
  Int32: "number",
  Int64: "string",
  Int128: "string",
  Int256: "string",
  Float32: "number",
  Float64: "number",
  Decimal: "number",
  Bool: "boolean",
  String: "string",
  FixedString: "string",
  UUID: "string",
  Date32: "string",
  Date64: "string",
  DateTime32: "string",
  DateTime64: "string",
  IPv4: "string",
  IPv6: "string",
};

export const mapFromClickHouseType = (clickHouseType: string) => {
  return typeMapping[clickHouseType] || "string";
};

/**
 * Convert the JS type (source is JSON format by API query parameter) to the corresponding ClickHouse type for generating named placeholder of parameterized query.
 * Only support to convert number to Int or Float, boolean to Bool, string to String, other types will convert to String.
 * If exist complex type e.g: object, Array, null, undefined, Date, Record.. etc, just convert to string type by ClickHouse function in SQL.
 * ClickHouse support converting string to other types function.
 * Please see Each section of the https://clickhouse.com/docs/en/sql-reference/functions and https://clickhouse.com/docs/en/sql-reference/functions/type-conversion-functions
 * @param value
 * @returns 'FLoat', 'Int', 'Bool', 'String'
 */

export const mapToClickHouseType = (value: any) => {
  if (typeof value === "number") {
    // infer the float or int according to exist remainder or not
    if (value % 1 !== 0) return "Float";
    return "Int";
  }
  // When define column type or query result with parameterized query, The Bool or Boolean type both supported.
  // But the column type of query result only return Bool, so we only support Bool type for safety.
  if (typeof value === "boolean") return "Bool";
  if (value instanceof Date) return "DateTime";
  return "String";
};

export function createClickhouseParameter(parameterIndex, value) {
  // ClickHouse use {name:type} be a placeholder, so if we only use number string as name e.g: {1:Unit8}
  // it will face issue when converting to the query params => {1: value1}, because the key is value not string type, so here add prefix "p" to avoid this issue.
  return `{p${parameterIndex}:${mapToClickHouseType(value)}}`;
}

// source https://github.com/blakeembrey/sql-template-tag/blob/main/src/index.ts
/**
 * Values supported by SQL engine.
 */
export type Value = unknown;

/**
 * Supported value or SQL instance.
 */
export type RawValue = Value | Sql;

/**
 * A SQL instance can be nested within each other to build SQL strings.
 */
export class Sql {
  readonly values: Value[];
  readonly strings: string[];

  constructor(rawStrings: readonly string[], rawValues: readonly RawValue[]) {
    if (rawStrings.length - 1 !== rawValues.length) {
      if (rawStrings.length === 0) {
        throw new TypeError("Expected at least 1 string");
      }

      throw new TypeError(
        `Expected ${rawStrings.length} strings to have ${
          rawStrings.length - 1
        } values`,
      );
    }

    const valuesLength = rawValues.reduce<number>(
      (len: number, value: RawValue) =>
        len + (value instanceof Sql ? value.values.length : 1),
      0,
    );

    this.values = new Array(valuesLength);
    this.strings = new Array(valuesLength + 1);

    this.strings[0] = rawStrings[0];

    // Iterate over raw values, strings, and children. The value is always
    // positioned between two strings, e.g. `index + 1`.
    let i = 0,
      pos = 0;
    while (i < rawValues.length) {
      const child = rawValues[i++];
      const rawString = rawStrings[i];

      // Check for nested `sql` queries.
      if (child instanceof Sql) {
        // Append child prefix text to current string.
        this.strings[pos] += child.strings[0];

        let childIndex = 0;
        while (childIndex < child.values.length) {
          this.values[pos++] = child.values[childIndex++];
          this.strings[pos] = child.strings[childIndex];
        }

        // Append raw string to current string.
        this.strings[pos] += rawString;
      } else {
        this.values[pos++] = child;
        this.strings[pos] = rawString;
      }
    }
  }
}

export default function sql(
  strings: readonly string[],
  ...values: readonly RawValue[]
) {
  return new Sql(strings, values);
}
