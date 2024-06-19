/**
 * Convert the JS type (source is JSON format by API query parameter) to the corresponding ClickHouse type for generating named placeholder of parameterized query.
 * Only support to convert number to Int or Float, boolean to Bool, string to String, other types will convert to String.
 * If exist complex type e.g: object, Array, null, undefined, Date, Record.. etc, just convert to string type by ClickHouse function in SQL.
 * ClickHouse support converting string to other types function.
 * Please see Each section of the https://clickhouse.com/docs/en/sql-reference/functions and https://clickhouse.com/docs/en/sql-reference/functions/type-conversion-functions
 * @param value
 * @returns 'FLoat', 'Int', 'Bool', 'String'
 */

import { ClickHouseClient } from "@clickhouse/client-web";

export const mapToClickHouseType = (value: Value) => {
  if (typeof value === "number") {
    // infer the float or int according to exist remainder or not
    if (value % 1 !== 0) return "Float";
    return "Int";
  }
  // When define column type or query result with parameterized query, The Bool or Boolean type both supported.
  // But the column type of query result only return Bool, so we only support Bool type for safety.
  if (typeof value === "boolean") return "Bool";
  if (value instanceof Date) return "DateTime";
  if (Array.isArray(value)) {
    const [type, _] = value;
    return type;
  }
  return "String";
};

export const getValueFromParameter = (value: any) => {
  if (Array.isArray(value)) {
    const [type, val] = value;
    if (type === "Identifier") return val;
  }
  return value;
};

export function createClickhouseParameter(
  parameterIndex: number,
  value: Value,
) {
  // ClickHouse use {name:type} be a placeholder, so if we only use number string as name e.g: {1:Unit8}
  // it will face issue when converting to the query params => {1: value1}, because the key is value not string type, so here add prefix "p" to avoid this issue.
  return `{p${parameterIndex}:${mapToClickHouseType(value)}}`;
}

// source https://github.com/blakeembrey/sql-template-tag/blob/main/src/index.ts
/**
 * Values supported by SQL engine.
 */
export type Value = string | number | boolean | Date | [string, string];

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

export function sql(
  strings: readonly string[],
  ...values: readonly RawValue[]
) {
  return new Sql(strings, values);
}

function emptyIfUndefined(value: string | undefined): string {
  return value === undefined ? "" : value;
}

export class MooseClient {
  client: ClickHouseClient;
  constructor(client: ClickHouseClient) {
    this.client = client;
  }

  async query(sql: Sql) {
    const parameterizedStubs = sql.values.map((v, i) =>
      createClickhouseParameter(i, v),
    );

    const query = sql.strings
      .map((s, i) =>
        s != "" ? `${s}${emptyIfUndefined(parameterizedStubs[i])}` : "",
      )
      .join("");

    const query_params = sql.values.reduce(
      (acc: Record<string, unknown>, v, i) => ({
        ...acc,
        [`p${i}`]: getValueFromParameter(v),
      }),
      {},
    );

    console.log("query", query);
    console.log("query_params", query_params);

    return this.client.query({
      query,
      query_params,
      format: "JSONEachRow",
    });
  }
}

export const ConsumptionHelpers = {
  column: (value: string) => ["Identifier", value] as [string, string],
  table: (value: string) => ["Identifier", value] as [string, string],
};

export function join_queries({
  values,
  separator = ",",
  prefix = "",
  suffix = "",
}: {
  values: readonly RawValue[];
  separator?: string;
  prefix?: string;
  suffix?: string;
}) {
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
