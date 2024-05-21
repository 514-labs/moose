import type { ClickHouseClient } from "npm:@clickhouse/client-web@1.0.1";

import { ResultSet } from "npm:@clickhouse/client-web@1.0.1";
import sql, { Sql, createClickhouseParameter } from "./consumption-helpers.ts";
import { antiCachePath, getClickhouseClient } from "./ts-helpers.ts";

const cwd = Deno.args[0] || Deno.cwd();
const CONSUMPTION_DIR_PATH = `${cwd}/app/apis`;

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

const apiHandler = async (request: Request): Promise<Response> => {
  const url = new URL(request.url);
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

  return new Response(body, { status: 200 });
};

export const startApiService = async () =>
  Deno.serve({ port: 4001 }, apiHandler);

startApiService();
