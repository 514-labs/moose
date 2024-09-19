import http from "http";
import process from "node:process";
import { getClickhouseClient } from "../commons";
import { MooseClient, sql } from "./helpers";

export const antiCachePath = (path: string) =>
  `${path}?num=${Math.random().toString()}&time=${Date.now()}`;

const [
  ,
  ,
  ,
  CONSUMPTION_DIR_PATH,
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
  try {
    const url = new URL(req.url || "", "https://localhost");
    const fileName = url.pathname;

    const pathName = createPath(fileName);

    const paramsObject = Array.from(url.searchParams.entries()).reduce(
      (obj: { [key: string]: any }, [key, value]) => {
        if (obj[key]) {
          if (Array.isArray(obj[key])) {
            obj[key].push(value);
          } else {
            obj[key] = [obj[key], value];
          }
        } else {
          obj[key] = value;
        }
        return obj;
      },
      {},
    );

    const userFuncModule = require(pathName);

    const result = await userFuncModule.default(paramsObject, {
      client: new MooseClient(getClickhouseClient(clickhouseConfig)),
      sql: sql,
    });

    let body: string;

    // TODO investigate why these prototypes are different
    if (Object.getPrototypeOf(result).constructor.name === "ResultSet") {
      body = JSON.stringify(await result.json());
    } else {
      body = JSON.stringify(result);
    }

    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(body);
  } catch (error: any) {
    if (error instanceof Error) {
      res.writeHead(500, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ error: error.message }));
    } else {
      res.writeHead(500, { "Content-Type": "application/json" });
      res;
    }
  }
};

export const runConsumptionApis = async () => {
  console.log("Starting API service");
  const server = http.createServer(apiHandler);

  process.on("SIGTERM", async () => {
    console.log("Received SIGTERM, shutting down...");
    server.close(() => {
      console.log("Consumption webserver shutdown...");
      process.exit(0);
    });
  });

  server.listen(4001, () => {
    console.log("Server running on port 4001");
  });
};
