// taken from https://github.com/Canner/vulcan-sql/blob/develop/packages/extension-driver-clickhouse/src/lib/typeMapper.ts

const typeMapping = new Map<string, string>();

const register = (clickHouseType: string, type: string) => {
  typeMapping.set(clickHouseType, type);
};

// Reference
// https://clickhouse.com/docs/en/native-protocol/columns#integers
// Currently, FieldDataType only support number, string, boolean, Date for generating response schema in the specification.
register("Int", "number");
register("UInt", "number");
register("UInt8", "number");
register("LowCardinality", "string");
register("UInt16", "number");
register("UInt32", "number");
register("UInt64", "string");
register("UInt128", "string");
register("UInt256", "string");
register("Int8", "number");
register("Int16", "number");
register("Int32", "number");
register("Int64", "string");
register("Int128", "string");
register("Int256", "string");
register("Float32", "number");
register("Float64", "number");
register("Decimal", "number");
// When define column type or query result with parameterized query, The Bool or Boolean type both supported.
// But the column type of query result only return Bool, so we only support Bool type for safety.
register("Bool", "boolean");
register("String", "string");
register("FixedString", "string");
register("UUID", "string");
register("Date32", "string");
register("Date64", "string");
register("DateTime32", "string");
register("DateTime64", "string");
register("IPv4", "string");
register("IPv6", "string");

export const mapFromClickHouseType = (clickHouseType: string) => {
  if (typeMapping.has(clickHouseType)) return typeMapping.get(clickHouseType)!;
  return "string";
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
  if (typeof value === "string") return "String";
  return "String";
};

export function createClickhouseParameter(parameterIndex, value) {
  // ClickHouse use {name:type} be a placeholder, so if we only use number string as name e.g: {1:Unit8}
  // it will face issue when converting to the query params => {1: value1}, because the key is value not string type, so here add prefix "p" to avoid this issue.
  return `{p${parameterIndex}:${mapToClickHouseType(value)}}`;
}

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

  get sql() {
    const len = this.strings.length;
    let i = 1;
    let value = this.strings[0];
    while (i < len) value += `?${this.strings[i++]}`;
    return value;
  }

  get statement() {
    const len = this.strings.length;
    let i = 1;
    let value = this.strings[0];
    while (i < len) value += `:${i}${this.strings[i++]}`;
    return value;
  }

  get text() {
    const len = this.strings.length;
    let i = 1;
    let value = this.strings[0];
    while (i < len) value += `$${i}${this.strings[i++]}`;
    return value;
  }

  inspect() {
    return {
      sql: this.sql,
      statement: this.statement,
      text: this.text,
      values: this.values,
    };
  }
}

/**
 * Create a SQL query for a list of values.
 */
export function join(
  values: readonly RawValue[],
  separator = ",",
  prefix = "",
  suffix = "",
) {
  if (values.length === 0) {
    throw new TypeError(
      "Expected `join([])` to be called with an array of multiple elements, but got an empty array",
    );
  }

  return new Sql(
    [prefix, ...Array(values.length - 1).fill(separator), suffix],
    values,
  );
}

/**
 * Create a SQL query for a list of structured values.
 */
export function bulk(
  data: ReadonlyArray<ReadonlyArray<RawValue>>,
  separator = ",",
  prefix = "",
  suffix = "",
) {
  const length = data.length && data[0].length;

  if (length === 0) {
    throw new TypeError(
      "Expected `bulk([][])` to be called with a nested array of multiple elements, but got an empty array",
    );
  }

  const values = data.map((item, index) => {
    if (item.length !== length) {
      throw new TypeError(
        `Expected \`bulk([${index}][])\` to have a length of ${length}, but got ${item.length}`,
      );
    }

    return new Sql(["(", ...Array(item.length - 1).fill(separator), ")"], item);
  });

  return new Sql(
    [prefix, ...Array(values.length - 1).fill(separator), suffix],
    values,
  );
}

/**
 * Create raw SQL statement.
 */
export function raw(value: string) {
  return new Sql([value], []);
}

/**
 * Placeholder value for "no text".
 */
export const empty = raw("");

/**
 * Create a SQL object from a template string.
 */
export default function sql(
  strings: readonly string[],
  ...values: readonly RawValue[]
) {
  return new Sql(strings, values);
}
