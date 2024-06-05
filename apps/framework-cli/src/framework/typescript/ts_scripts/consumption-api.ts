import {
  ClickHouseClient,
  ResultSet,
  createClient,
} from "@clickhouse/client-web";
import http from "http";
import process from "node:process";
import { createClickhouseParameter, Sql, sql } from "@514labs/moose-lib";

export const antiCachePath = (path: string) =>
  `${path}?num=${Math.random().toString()}&time=${Date.now()}`;

console.log("Starting consumption API");

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

const createPath = (path: string) => `${CONSUMPTION_DIR_PATH}${path}.ts`;

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

    return this.client.query({
      query,
      query_params,
      format: "JSONEachRow",
    });
  }
}

const apiHandler = async (req, res) => {
  const url = new URL(req.url, "https://localhost");
  const fileName = url.pathname;

  const pathName = createPath(fileName);

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

  server.listen(4001, () => {
    console.log("Server running on port 4001");
  });
};

startApiService();
