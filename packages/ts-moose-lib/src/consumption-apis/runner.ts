import http from "http";
import process from "node:process";
import { getClickhouseClient } from "../commons";
import { MooseClient, sql } from "./helpers";
import * as jose from "jose";
import { ClickHouseClient } from "@clickhouse/client";
import { Cluster } from "../cluster-utils";
import { QueryParamMetadata } from "./types";
import { QueryParamParser } from "./parser";
import { ConsumptionUtil } from "../index";
import typia, { tags } from "typia";
import { logToConsole } from "./hlogger";
import { ApiResponse } from "./types";
import { register } from "ts-node";
import { resolve } from "path";

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
  logToConsole(`httpLogger: ${req.method} ${req.url} ${res.statusCode}`);
};

const modulesCache = new Map<string, any>();

export interface ConsumptionApiConfig {
  params: QueryParamMetadata;
}

export function createConsumptionApi<T extends object, R = any>(
  handler: (params: T, utils: ConsumptionUtil) => Promise<R>,
) {
  let validator;
  let validate;

  try {
    logToConsole(`>>> ${handler}`);

    validator = typia.createIs<T>();
    validate = typia.createValidate<T>();
    logToConsole(
      `createConsumptionApi: validator: ${JSON.stringify(validator)}`,
    );
  } catch (error) {
    logToConsole(
      `createConsumptionApi: Fatal building validator: ${error instanceof Error ? error.message : String(error)}`,
    );
    throw error;
  }

  return async function (
    rawParams: Record<string, string[]>,
    utils: ConsumptionUtil,
  ) {
    const processedParams = Object.fromEntries(
      Object.entries(rawParams).map(([key, values]) => [
        key,
        values.length === 1
          ? isNaN(Number(values[0]))
            ? values[0]
            : Number(values[0])
          : values,
      ]),
    );

    try {
      logToConsole(
        `createConsumptionApi: Processed params: ${JSON.stringify(processedParams)}`,
      );
      if (!validator(processedParams)) {
        logToConsole(
          `createConsumptionApi: Validation error: Invalid parameters`,
        );
        throw new Error("Invalid parameters");
      }

      const result = await handler(processedParams as T, utils);

      return {
        status: 200,
        body: {
          success: true,
          data: result,
        },
      };
    } catch (error) {
      logToConsole(
        `createConsumptionApi: Validation error: ${error instanceof Error ? error.message : String(error)}`,
      );

      return {
        status: 400,
        body: {
          success: false,
          error: {
            code: "VALIDATION_ERROR",
            message:
              error instanceof Error ? error.message : "Invalid parameters",
            details: validate(processedParams),
          },
        },
      };
    }
  };
}

const apiHandler =
  (publicKey: jose.KeyLike | undefined, clickhouseClient: ClickHouseClient) =>
  async (req: http.IncomingMessage, res: http.ServerResponse) => {
    logToConsole(`apiHandler: Starting API handler`);
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
      logToConsole(`apiHandler: Path name: ${pathName}`);
      const paramsObject = Array.from(url.searchParams.entries()).reduce(
        (obj: { [key: string]: string[] }, [key, value]) => {
          if (obj[key]) {
            obj[key].push(value);
          } else {
            obj[key] = [value];
          }
          return obj;
        },
        {},
      );

      logToConsole(`apiHandler: paramsObject: ${JSON.stringify(paramsObject)}`);

      let userFuncModule;
      try {
        logToConsole(
          `apiHandler: About to register ts-node with typia transformer`,
        );
        register({
          transpileOnly: true,
          compiler: "ts-patch/compiler",
          compilerOptions: {
            plugins: [
              {
                transform: "typia/lib/transform",
                after: true,
                transformProgram: true,
              },
            ],
            experimentalDecorators: true,
          },
        });
        logToConsole(
          `apiHandler: Successfully registered ts-node with typia transformer`,
        );

        // const absolutePath = resolve(pathName);
        // logToConsole(`apiHandler: Loading module from: ${absolutePath}`);

        // Clear require cache for this file to ensure fresh load
        // delete require.cache[absolutePath];

        // logToConsole(`apiHandler: pre require: ${absolutePath}`);
        userFuncModule = require(pathName);
        // logToConsole(`apiHandler: post require: ${absolutePath}`);

        // if (!userFuncModule) {
        //   throw new Error(`Module at ${absolutePath} returned undefined`);
        // }

        logToConsole(`apiHandler: Module loaded successfully`);

        if (userFuncModule === undefined) {
          logToConsole(`apiHandler: Adding to modulesCache: ${pathName}`);
          modulesCache.set(pathName, userFuncModule);
        }
      } catch (error) {
        logToConsole(
          `apiHandler: Error registering ts-node: ${error instanceof Error ? error.stack : String(error)}`,
        );
        throw error;
      }

      // try {
      //   logToConsole(`apiHandler: About to load module from path: ${fileName}`);
      //   const fullPath = process.cwd() + fileName;
      //   logToConsole(`apiHandler: Full module path: ${fullPath}`);
      //
      //   // Log the module cache state
      //   logToConsole(
      //     `apiHandler: Module cache state before require: ${Object.keys(require.cache).join(", ")}`,
      //   );
      //
      //   userFuncModule = require(fullPath);
      //
      //   logToConsole(
      //     `apiHandler: Successfully loaded module. Exports: ${Object.keys(userFuncModule).join(", ")}`,
      //   );
      // } catch (error) {
      //   logToConsole(
      //     `apiHandler: Error loading module: ${
      //       error instanceof Error ? error.stack : String(error)
      //     }`,
      //   );
      //   throw error;
      // }

      try {
        logToConsole(`apiHandler: Calling userFuncModule`);
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

        let response: ApiResponse;

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
        logToConsole(`apiHandler: Response: ${JSON.stringify(response)}`);
      } catch (error: unknown) {
        logToConsole(`API Error: ${String(error)}`);

        let response: ApiResponse;

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
        logToConsole(`apiHandler: Response: ${JSON.stringify(response)}`);
        return;
      }
    } catch (error: any) {
      logToConsole(
        `Fatal error: ${error instanceof Error ? error.stack : String(error)}`,
      );
      const response: ApiResponse = {
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
  logToConsole("Starting API service");

  // Add logging for project configuration
  const fs = require("fs");
  const path = require("path");

  try {
    const projectRoot = process.cwd();
    logToConsole(`Project root: ${projectRoot}`);

    const tsconfigPath = path.join(projectRoot, "tsconfig.json");
    logToConsole(`Looking for tsconfig at: ${tsconfigPath}`);

    if (fs.existsSync(tsconfigPath)) {
      const tsconfig = JSON.parse(fs.readFileSync(tsconfigPath, "utf8"));
      logToConsole(`Found tsconfig.json: ${JSON.stringify(tsconfig, null, 2)}`);
    } else {
      logToConsole("No tsconfig.json found in project root");
    }
  } catch (error) {
    logToConsole(
      `Error checking project configuration: ${error instanceof Error ? error.message : String(error)}`,
    );
  }

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
