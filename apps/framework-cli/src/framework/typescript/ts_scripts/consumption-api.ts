// @ts-nocheck
import {
  ClickHouseClient,
  ResultSet,
  createClient,
} from "@clickhouse/client-web";
import http from "http";

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

export const antiCachePath = (path: string) =>
  `${path}?num=${Math.random().toString()}&time=${Date.now()}`;
const [
  ,
  CONSUMPTION_DIR_PATH,
  CLICKHOUSE_DB,
  CLICKHOUSE_HOST,
  CLICKHOUSE_PORT,
  CLICKHOUSE_USERNAME,
  CLICKHOUSE_PASSWORD,
  CLICKHOUSE_USE_SSL,
] = process.argv;

console.log(CONSUMPTION_DIR_PATH);
const getClickhouseClient = () => {
  const protocol =
    CLICKHOUSE_USE_SSL === "1" || CLICKHOUSE_USE_SSL.toLowerCase() === "true"
      ? "https"
      : "http";
  console.log(
    `Connecting to Clickhouse at ${protocol}://${CLICKHOUSE_HOST}:${CLICKHOUSE_PORT}`,
  );
  return createClient({
    url: `${protocol}://${CLICKHOUSE_HOST}:${CLICKHOUSE_PORT}`,
    username: CLICKHOUSE_USERNAME,
    password: CLICKHOUSE_PASSWORD,
    database: CLICKHOUSE_DB,
  });
};

const createPath = (path: string) => `${CONSUMPTION_DIR_PATH}/${path}.ts`;

function emptyIfUndefined(value: string | undefined): string {
  return value === undefined ? "" : value;
}

class MooseClient {
  client: ClickHouseClient;
  constructor() {
    this.client = getClickhouseClient();
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
      (acc: Record<string, unknown>, v, i) => ({ ...acc, [`p${i}`]: v }),
      {},
    );

    console.log("Querying Clickhouse with", query, query_params);

    return this.client.query({
      query,
      query_params,
      format: "JSONEachRow",
    });
  }
}

const apiHandler = async (req, res) => {
  const url = new URL(req.url);
  const fileName = url.pathname;

  const pathName = antiCachePath(createPath(fileName));

  const searchParams = Object.fromEntries(url.searchParams.entries());

  const userFuncModule = await import(pathName);

  const result = await userFuncModule.default(searchParams, {
    client: new MooseClient(),
    sql: sql,
  });

  let body: string;
  if (result instanceof ResultSet) {
    body = JSON.stringify(await result.json());
  } else {
    body = JSON.stringify(result);
  }

  res.writeHead(200, { "Content-Type": "application/json" });
  res.end(body);
};

const startApiService = async () => {
  console.log("Starting API service");
  const server = http.createServer(apiHandler);
  console.log("yo");

  server.listen(4001, () => {
    console.log("Server running on port 4001");
  });
};

startApiService();
