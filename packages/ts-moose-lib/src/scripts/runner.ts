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

const [, , , TEMPORAL_URL, SCRIPT_DIR, CLIENT_CERT, CLIENT_KEY, API_KEY] =
  process.argv;

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
  temporalUrl: string,
  clientCert: string,
  clientKey: string,
  apiKey: string,
): Promise<{ connection: NativeConnection; namespace: string }> {
  let namespace = "default";
  if (!temporalUrl.includes("localhost")) {
    // Remove port and just get <namespace>.<account>
    const hostPart = temporalUrl.split(":")[0];
    const match = hostPart.match(/^([^.]+\.[^.]+)/);
    if (match && match[1]) {
      namespace = match[1];
    }
  }
  logger.info(`Using namespace from URL: ${namespace}`);

  let connectionOptions: NativeConnectionOptions = {
    address: temporalUrl,
  };

  if (!temporalUrl.includes("localhost")) {
    // URL with mTLS uses gRPC namespace endpoint which is what temporalUrl already is
    if (clientCert && clientKey) {
      logger.info("Using TLS for non-local Temporal");
      const cert = await fs.readFileSync(clientCert);
      const key = await fs.readFileSync(clientKey);

      connectionOptions.tls = {
        clientCertPair: {
          crt: cert,
          key: key,
        },
      };
    } else if (apiKey) {
      logger.info(`Using API key for non-local Temporal`);
      // URL with API key uses gRPC regional endpoint
      connectionOptions.address = "us-west1.gcp.api.temporal.io:7233";
      connectionOptions.apiKey = apiKey;
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
): Promise<Worker | null> {
  logger.info(`Registering workflows from ${SCRIPT_DIR}`);

  // Collect all TypeScript scripts
  const allScriptPaths: string[] = [];

  try {
    // Process each workflow directory
    const workflowDirs = fs.readdirSync(SCRIPT_DIR);
    for (const workflowDir of workflowDirs) {
      const workflowDirFullPath = path.join(SCRIPT_DIR, workflowDir);
      logger.info(`Checking workflow directory: ${workflowDirFullPath}`);

      if (fs.statSync(workflowDirFullPath).isDirectory()) {
        allScriptPaths.push(...collectActivities(logger, workflowDirFullPath));
      }
    }

    if (allScriptPaths.length === 0) {
      logger.info(`No scripts found in ${SCRIPT_DIR}`);
      return null;
    }

    logger.info(`Found ${allScriptPaths.length} scripts in ${SCRIPT_DIR}`);

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
      logger.info(`No tasks found in ${SCRIPT_DIR}`);
      return null;
    }

    logger.info(`Found ${dynamicActivities.length} task(s) in ${SCRIPT_DIR}`);

    const { connection, namespace } = await createTemporalConnection(
      logger,
      TEMPORAL_URL,
      CLIENT_CERT,
      CLIENT_KEY,
      API_KEY,
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
 * @param scriptDir - Root directory containing TypeScript scripts to register as activities.
 *                   Scripts will be registered with activity names in the format "parent_dir/script_name".
 * @returns The started Temporal worker instance
 * @throws ValueError if no scripts are found to register
 */
export async function runScripts(): Promise<Worker | null> {
  // Not sure why temporal doesn't like importing the logger
  // so have to pass it around
  const logger = initializeLogger();

  logger.info(`Starting worker for script directory: ${SCRIPT_DIR}`);
  const worker = await registerWorkflows(logger);
  if (!worker) {
    const msg = `No scripts found to register in ${SCRIPT_DIR}`;
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
