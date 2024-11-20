import http from "http";
import process from "node:process";
import cluster from "node:cluster";
import { getClickhouseClient } from "../commons";
import { MooseClient, sql } from "./helpers";
import * as jose from "jose";
import { ClickHouseClient } from "@clickhouse/client-web";
import { Cluster } from "../cluster-utils";

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
  JWT_SECRET, // Optional we will need to bring a proper cli parsing tool to help to make sure this is more resilient. or make it one json object
  JWT_ISSUER, // Optional
  JWT_AUDIENCE, // Optional
  ENFORCE_ON_ALL_CONSUMPTIONS_APIS, // Optional
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

const httpLogger = (req: http.IncomingMessage, res: http.ServerResponse) => {
  console.log(`${req.method} ${req.url} ${res.statusCode}`);
};

const modulesCache = new Map<string, any>();

const apiHandler =
  (publicKey: jose.KeyLike | undefined, clickhouseClient: ClickHouseClient) =>
  async (req: http.IncomingMessage, res: http.ServerResponse) => {
    try {
      const url = new URL(req.url || "", "https://localhost");
      const fileName = url.pathname;

      let jwtPayload;
      if (publicKey && JWT_ISSUER && JWT_AUDIENCE) {
        const jwt = req.headers.authorization?.split(" ")[1]; // Bearer <token>
        if (jwt) {
          try {
            const { payload } = await jose.jwtVerify(jwt, publicKey, {
              issuer: JWT_ISSUER,
              audience: JWT_AUDIENCE,
            });
            jwtPayload = payload;
          } catch (error) {
            console.log("JWT verification failed");
            if (ENFORCE_ON_ALL_CONSUMPTIONS_APIS === "true") {
              res.writeHead(401, { "Content-Type": "application/json" });
              res.end(JSON.stringify({ error: "Unauthorized" }));
              httpLogger(req, res);
              return;
            }
          }
        } else if (ENFORCE_ON_ALL_CONSUMPTIONS_APIS === "true") {
          res.writeHead(401, { "Content-Type": "application/json" });
          res.end(JSON.stringify({ error: "Unauthorized" }));
          httpLogger(req, res);
          return;
        }
      } else if (ENFORCE_ON_ALL_CONSUMPTIONS_APIS === "true") {
        res.writeHead(401, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ error: "Unauthorized" }));
        httpLogger(req, res);
        return;
      }

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

      let userFuncModule = modulesCache.get(pathName);
      if (userFuncModule === undefined) {
        userFuncModule = require(pathName);
        modulesCache.set(pathName, userFuncModule);
      }

      const result = await userFuncModule.default(paramsObject, {
        client: new MooseClient(clickhouseClient, fileName),
        sql: sql,
        jwt: jwtPayload,
      });

      let body: string;
      let status: number | undefined;

      // TODO investigate why these prototypes are different
      if (Object.getPrototypeOf(result).constructor.name === "ResultSet") {
        body = JSON.stringify(await result.json());
      } else {
        if ("body" in result && "status" in result) {
          body = JSON.stringify(result.body);
          status = result.status;
        } else {
          body = JSON.stringify(result);
        }
      }

      if (status) {
        res.writeHead(status, { "Content-Type": "application/json" });
        httpLogger(req, res);
      } else {
        res.writeHead(200, { "Content-Type": "application/json" });
        httpLogger(req, res);
      }

      res.end(body);
    } catch (error: any) {
      console.log(error);
      if (error instanceof Error) {
        res.writeHead(500, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ error: error.message }));
        httpLogger(req, res);
      } else {
        res.writeHead(500, { "Content-Type": "application/json" });
        res.end();
        httpLogger(req, res);
      }
    }
  };

export const runConsumptionApis = async () => {
  console.log("Starting API service");

  const consumptionCluster = new Cluster({
    workerStart: async () => {
      const clickhouseClient = getClickhouseClient(clickhouseConfig);
      let publicKey: jose.KeyLike | undefined;
      if (JWT_SECRET) {
        console.log("Importing JWT public key...");
        publicKey = await jose.importSPKI(JWT_SECRET, "RS256");
      }

      const server = http.createServer(apiHandler(publicKey, clickhouseClient));
      server.listen(4001, () => {
        console.log("Server running on port 4001");
      });

      return server;
    },
    workerStop: async (server) => {
      return new Promise<void>((resolve) => {
        server.close(() => resolve());
      });
    },
  });

  consumptionCluster.start();
};
