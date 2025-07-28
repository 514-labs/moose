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
import { ConsumptionApi } from "../dmv2/sdk/consumptionApi";

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
  namespace: string;
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

const toClientConfig = (config: ClickhouseConfig) => ({
  ...config,
  useSSL: config.useSSL ? "true" : "false",
});

const createPath = (consumptionDir: string, path: string) => {
  const pathSegments = path.replace(/^\/+/, "").split("/");

  if (pathSegments.length > 1) {
    const endpoint = pathSegments[0];
    const version = pathSegments[1];

    const formattedVersion = version.replace(/\./g, "_");
    const versionedPath = `${consumptionDir}${endpoint}_v${formattedVersion}.ts`;

    try {
      require.resolve(versionedPath);
      return versionedPath;
    } catch (e) {
      const majorVersion = version.split(".")[0];
      const majorVersionPath = `${consumptionDir}${endpoint}_v${majorVersion}.ts`;

      try {
        require.resolve(majorVersionPath);
        return majorVersionPath;
      } catch (e) {
        return `${consumptionDir}${endpoint}.ts`;
      }
    }
  }

  return `${consumptionDir}${path}.ts`;
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
    { isDmv2 }: { isDmv2: boolean },
    getEgressApiInstances: () => Promise<Map<string, ConsumptionApi>>,
    modulesCache: Map<string, any>,
    consumptionDir: string,
    clickhouseClient: ClickHouseClient,
    temporalClient?: TemporalClient,
  ) =>
  async (req: http.IncomingMessage, res: http.ServerResponse) => {
    const url = new URL(req.url || "", `http://${req.headers.host}`);

    res.setHeader("Access-Control-Allow-Origin", "*");
    res.setHeader("Access-Control-Allow-Methods", "GET, POST");
    res.setHeader(
      "Access-Control-Allow-Headers",
      "Authorization, Content-Type, baggage, sentry-trace, traceparent, tracestate",
    );

    if (req.method === "OPTIONS") {
      res.writeHead(200);
      res.end();
      return;
    }

    try {
      if (req.method !== "GET") {
        res.writeHead(405, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ error: "Method not allowed" }));
        return;
      }

      const versionFromHeader = req.headers["x-moose-api-version"] as string;

      const fileName = url.pathname.replace(/^\/+/, "");
      const pathSegments = fileName.split("/");
      let versionFromPath: string | undefined;
      let endpointName = fileName;

      if (pathSegments.length >= 2) {
        endpointName = pathSegments[0];
        versionFromPath = pathSegments[1];
      }

      const requestVersion = versionFromPath || versionFromHeader;

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

      let cacheKey = endpointName;
      if (isDmv2 && requestVersion) {
        cacheKey = `${endpointName}_v${requestVersion}`;
      }

      let userFuncModule = modulesCache.get(cacheKey);
      if (userFuncModule === undefined) {
        if (isDmv2) {
          const egressApis = await getEgressApiInstances();

          if (requestVersion) {
            const versionedApiKey = `v${requestVersion}/${endpointName}`;
            userFuncModule = egressApis.get(versionedApiKey);
          } else {
            userFuncModule = egressApis.get(endpointName);

            if (!userFuncModule) {
              const availableVersions: string[] = [];
              for (const key of egressApis.keys()) {
                const versionMatch = key.match(
                  `^v(\\d+(?:\\.\\d+)*)\/${endpointName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}$`,
                );
                if (versionMatch) {
                  availableVersions.push(versionMatch[1]);
                }
              }

              if (availableVersions.length > 0) {
                availableVersions.sort((a, b) => {
                  const aParts = a.split(".").map(Number);
                  const bParts = b.split(".").map(Number);
                  for (
                    let i = 0;
                    i < Math.max(aParts.length, bParts.length);
                    i++
                  ) {
                    const aPart = aParts[i] || 0;
                    const bPart = bParts[i] || 0;
                    if (aPart !== bPart) {
                      return bPart - aPart;
                    }
                  }
                  return 0;
                });

                const highestVersion = availableVersions[0];
                const fallbackApiKey = `v${highestVersion}/${endpointName}`;
                userFuncModule = egressApis.get(fallbackApiKey);
              }
            }
          }

          modulesCache.set(cacheKey, userFuncModule);
        } else {
          const modulePath = createPath(consumptionDir, endpointName);

          try {
            const moduleExports = await import(modulePath);
            userFuncModule = moduleExports.default || moduleExports;
            modulesCache.set(cacheKey, userFuncModule);
          } catch (error) {
            res.writeHead(404, { "Content-Type": "application/json" });
            res.end(JSON.stringify({ error: "API not found" }));
            return;
          }
        }
      }

      if (!userFuncModule) {
        let errorMessage = `API not found: ${endpointName}`;
        if (requestVersion) {
          errorMessage += ` with version ${requestVersion}`;
        }
        res.writeHead(404, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ error: errorMessage }));
        return;
      }

      let result: any;
      if (
        isDmv2 &&
        userFuncModule &&
        typeof userFuncModule.getHandler === "function"
      ) {
        const handler = userFuncModule.getHandler();
        const queryClient = new QueryClient(
          clickhouseClient,
          "consumption-api-",
        );
        const mooseClient = new MooseClient(queryClient, temporalClient);
        const consumptionUtil: ConsumptionUtil = {
          client: mooseClient,
          sql: sql,
          jwt: undefined,
        };
        result = await handler(paramsObject, consumptionUtil);
      } else if (typeof userFuncModule === "function") {
        const legacyUtil: any = { user: undefined };
        result = await userFuncModule(paramsObject, legacyUtil);
      } else {
        throw new Error(
          `Invalid userFuncModule: expected function or ConsumptionApi with getHandler method, got ${typeof userFuncModule}`,
        );
      }

      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(JSON.stringify(result));
    } catch (error) {
      res.writeHead(500, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ error: "Internal server error" }));
    }
  };

export const runConsumptionApis = async (config: ConsumptionApisConfig) => {
  const consumptionCluster = new Cluster({
    maxWorkerCount: 1,
    workerStart: async () => {
      let temporalClient: TemporalClient | undefined;
      if (config.temporalConfig) {
        temporalClient = await getTemporalClient(
          config.temporalConfig.url,
          config.temporalConfig.namespace,
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
        publicKey = await jose.importSPKI(config.jwtConfig.secret, "RS256");
      }

      const modulesCache = new Map<string, any>();

      const getEgressApiInstances = async (): Promise<Map<string, any>> => {
        const { getEgressApis } = await import("../dmv2/internal");
        const handlerMap = await getEgressApis();

        const { getMooseInternal } = await import("../dmv2/internal");
        return getMooseInternal().egressApis;
      };

      const server = http.createServer(
        apiHandler(
          { isDmv2: config.isDmv2 },
          getEgressApiInstances,
          modulesCache,
          config.consumptionDir,
          clickhouseClient,
          temporalClient,
        ),
      );
      const port = config.proxyPort !== undefined ? config.proxyPort : 4001;
      server.listen(port, "localhost");

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
