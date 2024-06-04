import {
  ClickHouseClient,
  ResultSet,
  createClient,
} from "@clickhouse/client-web";
import http from "http";

import sql, { Sql, createClickhouseParameter } from "./consumption-helpers.ts";
import { antiCachePath } from "./ts-helpers.ts";

const [
  ,
  _,
  CLICKHOUSE_DB,
  CLICKHOUSE_HOST,
  CLICKHOUSE_PORT,
  CLICKHOUSE_USERNAME,
  CLICKHOUSE_PASSWORD,
  CLICKHOUSE_USE_SSL,
] = process.argv;

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

export const startApiService = async () => {
  const server = http.createServer(apiHandler);

  server.listen(4001, () => {
    console.log("Server running on port 4001");
  });
};

startApiService();
