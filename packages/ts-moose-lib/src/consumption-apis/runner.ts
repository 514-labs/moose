import http from "http";
import { getClickhouseClient } from "../commons";
import { MooseClient, QueryClient, getTemporalClient } from "./helpers";
import * as jose from "jose";
import { ClickHouseClient } from "@clickhouse/client";
import { Cluster } from "../cluster-utils";
import { ConsumptionUtil } from "../index";
import { sql } from "../sqlHelpers";
import { Client as TemporalClient } from "@temporalio/client";
import { getEgressApis } from "../dmv2/internal";

interface ClickhouseConfig {
  database: string;
  host: string;
  port: string;
  username: string;
  password: string;
  useSSL: boolean;
}

interface JwtConfig {
  secret?: string;
  issuer: string;
  audience: string;
}

interface TemporalConfig {
  url: string;
  clientCert: string;
  clientKey: string;
  apiKey: string;
}

interface ConsumptionApisConfig {
  consumptionDir: string;
  clickhouseConfig: ClickhouseConfig;
  jwtConfig?: JwtConfig;
  temporalConfig?: TemporalConfig;
  enforceAuth: boolean;
  isDmv2: boolean;
  proxyPort?: number;
}

// Convert our config to Clickhouse client config
const toClientConfig = (config: ClickhouseConfig) => ({
  ...config,
  useSSL: config.useSSL ? "true" : "false",
});

const createPath = (consumptionDir: string, path: string) => {
  // Check if the path has version information
  const pathSegments = path.replace(/^\/+/, "").split("/");

  // If it's a versioned path like "v1/endpoint"
  if (pathSegments.length > 1 && pathSegments[0].startsWith("v")) {
    const version = pathSegments[0].substring(1); // Remove 'v' prefix
    const endpoint = pathSegments[1];

    // Try different version formats
    // 1. endpoint_v1_0_0 (full version)
    const formattedVersion = version.replace(/\./g, "_");
    const versionedPath = `${consumptionDir}${endpoint}_v${formattedVersion}.ts`;

    try {
      // Check if file exists
      require.resolve(versionedPath);
      return versionedPath;
    } catch (e) {
      // 2. Try endpoint_v1 (major version only)
      const majorVersion = version.split(".")[0];
      const majorVersionPath = `${consumptionDir}${endpoint}_v${majorVersion}.ts`;

      try {
        require.resolve(majorVersionPath);
        return majorVersionPath;
      } catch (e) {
        // Fall back to regular path
        return `${consumptionDir}${endpoint}.ts`;
      }
    }
  }

  // Regular unversioned path
  return `${consumptionDir}${path}.ts`;
};

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
  (
    publicKey: jose.KeyLike | undefined,
    clickhouseClient: ClickHouseClient,
    temporalClient: TemporalClient | undefined,
    consumptionDir: string,
    enforceAuth: boolean,
    isDmv2: boolean,
    jwtConfig?: JwtConfig,
  ) =>
  async (req: http.IncomingMessage, res: http.ServerResponse) => {
    try {
      const url = new URL(req.url || "", "http://localhost");
      const fileName = url.pathname;

      let jwtPayload;
      if (publicKey && jwtConfig) {
        const jwt = req.headers.authorization?.split(" ")[1]; // Bearer <token>
        if (jwt) {
          try {
            const { payload } = await jose.jwtVerify(jwt, publicKey, {
              issuer: jwtConfig.issuer,
              audience: jwtConfig.audience,
            });
            jwtPayload = payload;
          } catch (error) {
            console.log("JWT verification failed");
            if (enforceAuth) {
              res.writeHead(401, { "Content-Type": "application/json" });
              res.end(JSON.stringify({ error: "Unauthorized" }));
              httpLogger(req, res);
              return;
            }
          }
        } else if (enforceAuth) {
          res.writeHead(401, { "Content-Type": "application/json" });
          res.end(JSON.stringify({ error: "Unauthorized" }));
          httpLogger(req, res);
          return;
        }
      } else if (enforceAuth) {
        res.writeHead(401, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ error: "Unauthorized" }));
        httpLogger(req, res);
        return;
      }

      const pathName = createPath(consumptionDir, fileName);
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
        if (isDmv2) {
          const egressApis = await getEgressApis();
          const fileName = url.pathname.replace(/^\/+/, "");

          // Handle versioned paths (v1/endpoint) for DMv2
          const pathSegments = fileName.split("/");

          if (pathSegments.length > 1 && pathSegments[0].startsWith("v")) {
            // For versioned paths, first look for the fully qualified version name
            const versionedName = `${pathSegments[0]}/${pathSegments[1]}`;
            userFuncModule = egressApis.get(versionedName);

            if (!userFuncModule) {
              // Then try with just the endpoint name
              userFuncModule = egressApis.get(pathSegments[1]);
            }
          } else {
            // Regular unversioned path
            userFuncModule = egressApis.get(fileName);
          }

          modulesCache.set(pathName, userFuncModule);
        } else {
          try {
            userFuncModule = require(pathName);
            modulesCache.set(pathName, userFuncModule);
          } catch (e) {
            res.writeHead(404, { "Content-Type": "application/json" });
            res.end(JSON.stringify({ error: `API not found: ${fileName}` }));
            httpLogger(req, res);
            return;
          }
        }
      }

      const queryClient = new QueryClient(clickhouseClient, fileName);

      if (!userFuncModule) {
        res.writeHead(404, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ error: `API not found: ${fileName}` }));
        httpLogger(req, res);
        return;
      }

      let result =
        isDmv2 && userFuncModule.getHandler ?
          await userFuncModule.getHandler()(paramsObject, {
            client: new MooseClient(queryClient, temporalClient),
            sql: sql,
            jwt: jwtPayload,
          })
        : await userFuncModule.default(paramsObject, {
            client: new MooseClient(queryClient, temporalClient),
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
      console.log("error in path ", req.url, error);
      // todo: same workaround as ResultSet
      if (Object.getPrototypeOf(error).constructor.name === "TypeGuardError") {
        res.writeHead(400, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ error: error.message }));
        httpLogger(req, res);
      }
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

export const runConsumptionApis = async (config: ConsumptionApisConfig) => {
  const consumptionCluster = new Cluster({
    workerStart: async () => {
      let temporalClient: TemporalClient | undefined;
      if (config.temporalConfig) {
        temporalClient = await getTemporalClient(
          config.temporalConfig.url,
          config.temporalConfig.clientCert,
          config.temporalConfig.clientKey,
          config.temporalConfig.apiKey,
        );
      }
      const clickhouseClient = getClickhouseClient(
        toClientConfig(config.clickhouseConfig),
      );
      let publicKey: jose.KeyLike | undefined;
      if (config.jwtConfig?.secret) {
        console.log("Importing JWT public key...");
        publicKey = await jose.importSPKI(config.jwtConfig.secret, "RS256");
      }

      const server = http.createServer(
        apiHandler(
          publicKey,
          clickhouseClient,
          temporalClient,
          config.consumptionDir,
          config.enforceAuth,
          config.isDmv2,
          config.jwtConfig,
        ),
      );
      // port is now passed via config.proxyPort or defaults to 4001
      const port = config.proxyPort !== undefined ? config.proxyPort : 4001;
      server.listen(port, "localhost", () => {
        console.log(`Server running on port ${port}`);
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
