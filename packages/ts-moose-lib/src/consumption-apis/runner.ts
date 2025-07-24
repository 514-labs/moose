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

  // If it's a versioned path like "endpoint/1"
  if (pathSegments.length > 1) {
    const endpoint = pathSegments[0];
    const version = pathSegments[1];

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

      // Extract version information early and maintain it consistently
      // Check for version in headers (passed by Rust proxy)
      const versionFromHeader = req.headers["x-moose-api-version"] as string;

      // Parse version from URL path (e.g., "bar/1" -> version="1", endpoint="bar")
      const fileName = url.pathname.replace(/^\/+/, "");
      const pathSegments = fileName.split("/");
      let versionFromPath: string | undefined;
      let endpointName = fileName;

      if (pathSegments.length >= 2) {
        endpointName = pathSegments[0];
        versionFromPath = pathSegments[1];
      }

      // Use version from path first, then fallback to header
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

      // Create version-specific cache key for DMv2
      let cacheKey = endpointName;
      if (isDmv2 && requestVersion) {
        cacheKey = `${endpointName}_v${requestVersion}`;
      }

      let userFuncModule = modulesCache.get(cacheKey);
      if (userFuncModule === undefined) {
        if (isDmv2) {
          const egressApis = await getEgressApiInstances();

          // Debug logging
          console.log(
            `[DEBUG] Looking for endpoint: ${endpointName}, requestVersion: ${requestVersion}`,
          );
          console.log(`[DEBUG] Available APIs:`, Array.from(egressApis.keys()));

          // Improved deterministic version resolution logic
          if (requestVersion) {
            // When a specific version is requested, only look for that exact version
            const versionedApiKey = `v${requestVersion}/${endpointName}`;
            console.log(
              `[DEBUG] Looking for versioned API key: ${versionedApiKey}`,
            );
            userFuncModule = egressApis.get(versionedApiKey);

            if (!userFuncModule) {
              // Version was explicitly requested but not found - this is an error condition
              console.warn(
                `Explicitly requested version ${requestVersion} for endpoint ${endpointName} not found`,
              );
            }
          } else {
            // When no version is specified, try unversioned first, then fall back to any available version
            console.log(
              `[DEBUG] Looking for unversioned API key: ${endpointName}`,
            );
            userFuncModule = egressApis.get(endpointName);

            if (!userFuncModule) {
              // Try to find any versioned implementation as fallback
              // Use a more deterministic approach: sort versions and pick the highest one
              const availableVersions: string[] = [];
              for (const apiKey of egressApis.keys()) {
                const versionMatch = apiKey.match(
                  `^v(\\d+(?:\\.\\d+)*)\/${endpointName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}$`,
                );
                if (versionMatch) {
                  availableVersions.push(versionMatch[1]);
                }
              }

              if (availableVersions.length > 0) {
                // Sort versions and pick the highest one for deterministic behavior
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
                      return bPart - aPart; // Descending order (highest first)
                    }
                  }
                  return 0;
                });

                const highestVersion = availableVersions[0];
                const fallbackApiKey = `v${highestVersion}/${endpointName}`;
                userFuncModule = egressApis.get(fallbackApiKey);

                if (userFuncModule) {
                  console.info(
                    `No unversioned endpoint found for ${endpointName}, using version ${highestVersion} as fallback`,
                  );
                }
              }
            }
          }

          modulesCache.set(cacheKey, userFuncModule);
        } else {
          // Legacy module loading for non-DMv2
          const modulePath = createPath(consumptionDir, endpointName);

          try {
            const moduleExports = await import(modulePath);
            userFuncModule = moduleExports.default || moduleExports;
            modulesCache.set(cacheKey, userFuncModule);
          } catch (error) {
            console.error(`Failed to load module at ${modulePath}:`, error);
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
      // Check if it's a ConsumptionApi by looking for the getHandler method
      if (
        isDmv2 &&
        userFuncModule &&
        typeof userFuncModule.getHandler === "function"
      ) {
        // Handle DMv2 ConsumptionApi
        const handler = userFuncModule.getHandler();
        // Create proper ConsumptionUtil with database clients
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
        // Handle legacy function
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
      console.error("Error in API handler:", error);
      res.writeHead(500, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ error: "Internal server error" }));
    }
  };

export const runConsumptionApis = async (config: ConsumptionApisConfig) => {
  const consumptionCluster = new Cluster({
    // Fix: Use only 1 worker for consumption APIs to avoid port binding conflicts
    // Multiple workers don't provide significant benefits for I/O-bound consumption APIs
    // and cause issues when all workers try to bind to the same port
    maxWorkerCount: 1,
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

      const modulesCache = new Map<string, any>();

      // Get the egress APIs directly from the internal registry
      const getEgressApiInstances = async (): Promise<Map<string, any>> => {
        const { getEgressApis } = await import("../dmv2/internal");
        const handlerMap = await getEgressApis();

        // Convert handler map back to ConsumptionApi instances
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
