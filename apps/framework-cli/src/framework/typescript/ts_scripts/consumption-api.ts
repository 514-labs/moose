import { ResultSet } from "@clickhouse/client-web";
import http from "http";
import process from "node:process";
import { getClickhouseClient, MooseClient, sql } from "@514labs/moose-lib";

export const antiCachePath = (path: string) =>
  `${path}?num=${Math.random().toString()}&time=${Date.now()}`;

const CONSUMPTION_DIR_PATH = process.argv[1];

const [
  ,
  ,
  CLICKHOUSE_DB,
  CLICKHOUSE_HOST,
  CLICKHOUSE_PORT,
  CLICKHOUSE_USERNAME,
  CLICKHOUSE_PASSWORD,
  CLICKHOUSE_USE_SSL,
] = process.argv;

const clickhouseConfig = {
  username: CLICKHOUSE_USERNAME,
  password: CLICKHOUSE_PASSWORD,
  database: CLICKHOUSE_DB,
  useSSL: CLICKHOUSE_USE_SSL,
  host: CLICKHOUSE_HOST,
  port: CLICKHOUSE_PORT,
};

const createPath = (path: string) => `${CONSUMPTION_DIR_PATH}${path}.ts`;

const apiHandler = async (
  req: http.IncomingMessage,
  res: http.ServerResponse,
) => {
  const url = new URL(req.url || "", "https://localhost");
  const fileName = url.pathname;

  const pathName = createPath(fileName);

  const searchParams = Object.fromEntries(url.searchParams.entries());

  const userFuncModule = await import(pathName);

  const result = await userFuncModule.default(searchParams, {
    client: new MooseClient(getClickhouseClient(clickhouseConfig)),
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
