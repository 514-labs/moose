import {
  DefaultLogger,
  NativeConnection,
  NativeConnectionOptions,
  Worker,
} from "@temporalio/worker";
import * as path from "path";
import * as fs from "fs";
import { createActivityForScript } from "./activity";
import { activities } from "./activity";
import { initializeLogger } from "./logger";

interface TemporalConfig {
  url: string;
  clientCert?: string;
  clientKey?: string;
  apiKey?: string;
}

interface ScriptsConfig {
  scriptDir: string;
  temporalConfig: TemporalConfig;
}

// Maintain a global set of activity names we've already registered
const ALREADY_REGISTERED = new Set<string>();

const EXCLUDE_DIRS = [".moose"];

function collectActivities(
  logger: DefaultLogger,
  workflowDir: string,
): string[] {
  logger.info(`Collecting tasks from ${workflowDir}`);
  const scriptPaths: string[] = [];

  function walkDir(dir: string) {
    const files = fs.readdirSync(dir);

    // Skip excluded directories
    if (EXCLUDE_DIRS.some((excluded) => dir.includes(excluded))) {
      logger.info(`Skipping excluded directory: ${dir}`);
      return;
    }

    // Sort files to ensure consistent activity registration order
    files.sort().forEach((file) => {
      const fullPath = path.join(dir, file);
      const stat = fs.statSync(fullPath);

      if (stat.isDirectory()) {
        walkDir(fullPath);
      } else if (file.endsWith(".ts")) {
        scriptPaths.push(fullPath);
        logger.info(`Found script: ${fullPath}`);
      }
    });
  }

  walkDir(workflowDir);
  return scriptPaths;
}

/**
 * This looks similar to the client in apis.
 * Temporal SDK uses similar looking connection options & client,
 * but there are different libraries for a worker like this & a client
 * like in the apis.
 */
async function createTemporalConnection(
  logger: DefaultLogger,
  temporalConfig: TemporalConfig,
): Promise<{ connection: NativeConnection; namespace: string }> {
  let namespace = "default";
  if (!temporalConfig.url.includes("localhost")) {
    // Remove port and just get <namespace>.<account>
    const hostPart = temporalConfig.url.split(":")[0];
    const match = hostPart.match(/^([^.]+\.[^.]+)/);
    if (match && match[1]) {
      namespace = match[1];
    }
  }
  logger.info(`Using namespace from URL: ${namespace}`);

  let connectionOptions: NativeConnectionOptions = {
    address: temporalConfig.url,
  };

  if (!temporalConfig.url.includes("localhost")) {
    // URL with mTLS uses gRPC namespace endpoint which is what temporalUrl already is
    if (temporalConfig.clientCert && temporalConfig.clientKey) {
      logger.info("Using TLS for non-local Temporal");
      const cert = await fs.readFileSync(temporalConfig.clientCert);
      const key = await fs.readFileSync(temporalConfig.clientKey);

      connectionOptions.tls = {
        clientCertPair: {
          crt: cert,
          key: key,
        },
      };
    } else if (temporalConfig.apiKey) {
      logger.info(`Using API key for non-local Temporal`);
      // URL with API key uses gRPC regional endpoint
      connectionOptions.address = "us-west1.gcp.api.temporal.io:7233";
      connectionOptions.apiKey = temporalConfig.apiKey;
      connectionOptions.tls = {};
      connectionOptions.metadata = {
        "temporal-namespace": namespace,
      };
    } else {
      logger.error("No authentication credentials provided for Temporal.");
    }
  }

  const connection = await NativeConnection.connect(connectionOptions);
  return { connection, namespace };
}

async function registerWorkflows(
  logger: DefaultLogger,
  config: ScriptsConfig,
): Promise<Worker | null> {
  logger.info(`Registering workflows from ${config.scriptDir}`);

  // Collect all TypeScript scripts
  const allScriptPaths: string[] = [];

  try {
    // Process each workflow directory
    const workflowDirs = fs.readdirSync(config.scriptDir);
    for (const workflowDir of workflowDirs) {
      const workflowDirFullPath = path.join(config.scriptDir, workflowDir);
      logger.info(`Checking workflow directory: ${workflowDirFullPath}`);

      if (fs.statSync(workflowDirFullPath).isDirectory()) {
        allScriptPaths.push(...collectActivities(logger, workflowDirFullPath));
      }
    }

    if (allScriptPaths.length === 0) {
      logger.info(`No scripts found in ${config.scriptDir}`);
      return null;
    }

    logger.info(
      `Found ${allScriptPaths.length} scripts in ${config.scriptDir}`,
    );

    // Build dynamic activities
    const dynamicActivities: any[] = [];
    for (const scriptPath of allScriptPaths) {
      const parentDir = path.basename(path.dirname(scriptPath));
      const baseName = path.basename(scriptPath, path.extname(scriptPath));
      const activityName = `${parentDir}/${baseName}`;

      if (!ALREADY_REGISTERED.has(activityName)) {
        const activity = await createActivityForScript(activityName);
        dynamicActivities.push(activity);
        ALREADY_REGISTERED.add(activityName);
        logger.info(`Registered task ${activityName}`);
      }
    }

    if (dynamicActivities.length === 0) {
      logger.info(`No tasks found in ${config.scriptDir}`);
      return null;
    }

    logger.info(
      `Found ${dynamicActivities.length} task(s) in ${config.scriptDir}`,
    );

    const { connection, namespace } = await createTemporalConnection(
      logger,
      config.temporalConfig,
    );

    const worker = await Worker.create({
      connection,
      namespace: namespace,
      taskQueue: "typescript-script-queue",
      workflowsPath: path.resolve(__dirname, "scripts/workflow.js"),
      activities: {
        ...activities,
        ...Object.fromEntries(
          dynamicActivities.map((activity) => [
            Object.keys(activity)[0],
            Object.values(activity)[0],
          ]),
        ),
      },
    });

    return worker;
  } catch (error) {
    logger.error(`Error registering workflows: ${error}`);
    throw error;
  }
}

/**
 * Start a Temporal worker that handles TypeScript script execution workflows.
 *
 * @param config - Configuration object containing script directory and temporal settings
 * @returns The started Temporal worker instance
 * @throws ValueError if no scripts are found to register
 */
export async function runScripts(
  config: ScriptsConfig,
): Promise<Worker | null> {
  // Not sure why temporal doesn't like importing the logger
  // so have to pass it around
  const logger = initializeLogger();

  // TODO: Remove this
  console.log(config);

  logger.info(`Starting worker for script directory: ${config.scriptDir}`);
  const worker = await registerWorkflows(logger, config);
  if (!worker) {
    const msg = `No scripts found to register in ${config.scriptDir}`;
    logger.warn(msg);
    return null;
  }

  logger.info("Starting TypeScript worker...");
  try {
    await worker.run();
  } catch (error) {
    logger.error(`Worker failed to start: ${error}`);
    throw error;
  }

  return worker;
}
