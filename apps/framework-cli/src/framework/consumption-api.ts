import type { ClickHouseClient } from "npm:@clickhouse/client-web@1.0.1";

import { createClient, ResultSet } from "npm:@clickhouse/client-web@1.0.1";
import sql, { Sql, createClickhouseParameter } from "./consumption-helpers.ts";

const CLICKHOUSE_DB =
  Deno.env.get("MOOSE_CLICKHOUSE_CONFIG__DB_NAME") || "local";
const CLICKHOUSE_HOST =
  Deno.env.get("MOOSE_CLICKHOUSE_CONFIG__HOST") || "localhost";
const CLICKHOUSE_PORT =
  Deno.env.get("MOOSE_CLICKHOUSE_CONFIG__HOST_PORT") || "18123";
const CLICKHOUSE_USERNAME =
  Deno.env.get("MOOSE_CLICKHOUSE_CONFIG__USER") || "panda";
const CLICKHOUSE_PASSWORD =
  Deno.env.get("MOOSE_CLICKHOUSE_CONFIG__PASSWORD") || "pandapass";
const CLICKHOUSE_USE_SSL =
  Deno.env.get("MOOSE_CLICKHOUSE_CONFIG__USE_SSL") || "false";

const getClickhouseClient = () => {
  const protocol =
    CLICKHOUSE_USE_SSL.toLowerCase() === "true" ? "https" : "http";
  console.log(
    `Connecting to Clickhouse at ${protocol}://${CLICKHOUSE_HOST}:${CLICKHOUSE_PORT}`,
  );
  return createClient({
    host: `${protocol}://${CLICKHOUSE_HOST}:${CLICKHOUSE_PORT}`,
    username: CLICKHOUSE_USERNAME,
    password: CLICKHOUSE_PASSWORD,
    database: CLICKHOUSE_DB,
  });
};

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

let i = 0;

const apiHandler = async (request: Request): Promise<Response> => {
  const path = new URL(request.url);
  const pathname = path.pathname;

  const searchParams = Object.fromEntries(path.searchParams.entries());

  const userFuncModule = await import(
    // the path is different every time so it reloads
    `/apis${pathname}.ts?import_trigger=${i++}`
  );

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
  return new Response(body, { status: 200 });
};

export const startApiService = async () =>
  Deno.serve({ port: 4001 }, apiHandler);

startApiService();
