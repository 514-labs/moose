import http from "http";
import process from "node:process";
import { getClickhouseClient } from "../commons";
import { MooseClient, sql } from "./helpers";
import * as jose from "jose";
import { ClickHouseClient } from "@clickhouse/client";
import { Cluster } from "../cluster-utils";
import { ConsumptionUtil } from "../index";

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

export function createConsumptionApi<T extends object, R = any>(
  _handler: (params: T, utils: ConsumptionUtil) => Promise<R>,
): (
  rawParams: Record<string, string[] | string>,
  utils: ConsumptionUtil,
) => Promise<R> {
  throw new Error(
    "This should be compiled-time replaced by compiler plugins to add parsing.",
  );
}

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
        (obj: { [key: string]: string[] | string }, [key, value]) => {
          const existingValue = obj[key];
          if (existingValue) {
            if (Array.isArray(existingValue)) {
              existingValue.push(value);
            } else {
              obj[key] = [existingValue, value];
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
        try {
          userFuncModule = require(pathName);
          modulesCache.set(pathName, userFuncModule);
        } catch (error) {
          console.log("error loading user's module", error);
          throw error;
        }
      }

      try {
        const result = await userFuncModule.default(paramsObject, {
          client: new MooseClient(clickhouseClient, fileName),
          sql: sql,
          jwt: jwtPayload,
        });

        // If result is already an API response shape, return it as is
        if ("status" in result && "body" in result) {
          res.statusCode = result.status;
          res.setHeader("Content-Type", "application/json");
          res.end(JSON.stringify(result.body));
          return;
        }

        let response: any;

        if (Object.getPrototypeOf(result).constructor.name === "ResultSet") {
          response = {
            success: true,
            data: await result.json(),
          };
        } else if ("body" in result && "status" in result) {
          // Handle an existing custom response format
          response = {
            success: result.status < 400,
            ...(result.status < 400
              ? { data: result.body }
              : { error: result.body }),
          };
          res.statusCode = result.status;
        } else {
          response = {
            success: true,
            data: result,
          };
        }

        res.setHeader("Content-Type", "application/json");
        res.end(JSON.stringify(response));
      } catch (error: unknown) {
        console.log(error);

        let response: any;

        if (
          error instanceof Error &&
          (error as any).type === "VALIDATION_ERROR"
        ) {
          response = {
            success: false,
            error: {
              code: "VALIDATION_ERROR",
              message: error.message,
            },
          };
          res.statusCode = 400;
        } else {
          response = {
            success: false,
            error: {
              code: "INTERNAL_ERROR",
              message:
                error instanceof Error
                  ? error.message
                  : "An unexpected error occurred",
              details:
                process.env.NODE_ENV === "development" ? error : undefined,
            },
          };
          res.statusCode = 500;
        }

        res.setHeader("Content-Type", "application/json");
        res.end(JSON.stringify(response));
        return;
      }
    } catch (error: any) {
      const response: any = {
        success: false,
        error: {
          code: "SYSTEM_ERROR",
          message: "A system error occurred",
          details:
            process.env.NODE_ENV === "development" ? error.message : undefined,
        },
      };
      res.statusCode = 500;
      res.setHeader("Content-Type", "application/json");
      res.end(JSON.stringify(response));
      httpLogger(req, res);
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
