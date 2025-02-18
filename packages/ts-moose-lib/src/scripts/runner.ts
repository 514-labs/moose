import { DefaultLogger, NativeConnection, Worker } from "@temporalio/worker";
import * as path from "path";
import * as fs from "fs";
import { createActivityForScript } from "./activity";
import { activities } from "./activity";
import { initializeLogger } from "./logger";

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

async function registerWorkflows(
  logger: DefaultLogger,
  scriptDir: string,
): Promise<Worker | null> {
  logger.info(`Registering workflows from ${scriptDir}`);

  // Collect all TypeScript scripts
  const allScriptPaths: string[] = [];

  try {
    // Process each workflow directory
    const workflowDirs = fs.readdirSync(scriptDir);
    for (const workflowDir of workflowDirs) {
      const workflowDirFullPath = path.join(scriptDir, workflowDir);
      logger.info(`Checking workflow directory: ${workflowDirFullPath}`);

      if (fs.statSync(workflowDirFullPath).isDirectory()) {
        allScriptPaths.push(...collectActivities(logger, workflowDirFullPath));
      }
    }

    if (allScriptPaths.length === 0) {
      logger.info(`No scripts found in ${scriptDir}`);
      return null;
    }

    logger.info(`Found ${allScriptPaths.length} scripts in ${scriptDir}`);

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
      logger.info(`No tasks found in ${scriptDir}`);
      return null;
    }

    logger.info(`Found ${dynamicActivities.length} task(s) in ${scriptDir}`);

    // TODO: Make this configurable
    logger.info("Connecting to Temporal server...");
    const connection = await NativeConnection.connect({
      address: "localhost:7233",
    });

    const worker = await Worker.create({
      connection,
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
export async function runScripts(scriptDir: string): Promise<Worker | null> {
  // Not sure why temporal doesn't like importing the logger
  // so have to pass it around
  const logger = initializeLogger();

  logger.info(`Starting worker for script directory: ${scriptDir}`);
  const worker = await registerWorkflows(logger, scriptDir);

  if (!worker) {
    const msg = `No scripts found to register in ${scriptDir}`;
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
