import { log as logger, proxyActivities } from "@temporalio/workflow";
import { WorkflowState } from "./types";
import { mooseJsonEncode } from "./serialization";

const { executeScript, readDirectory } = proxyActivities({
  startToCloseTimeout: "10 minutes",
  retry: {
    // TODO: Use user's retry config
    maximumAttempts: 3,
  },
});

export async function ScriptWorkflow(
  path: string,
  inputData?: any,
): Promise<any[]> {
  const state: WorkflowState = {
    completedSteps: [],
    currentStep: null,
    failedStep: null,
    scriptPath: null,
    inputData: null,
  };

  const results: any[] = [];
  let currentData = inputData?.data || inputData || {};

  logger.info(
    `Starting workflow for ${path} with data: ${JSON.stringify(currentData)}`,
  );

  try {
    // Encode input data
    currentData = JSON.parse(mooseJsonEncode({ data: currentData }));

    if (path.endsWith(".ts")) {
      const result = await executeScript({
        scriptPath: path,
        inputData: currentData,
      });
      return [result];
    }

    // Sequential execution
    const items = await readDirectory(path);
    for (const item of items.sort()) {
      const fullPath = `${path}/${item}`;

      if (item.endsWith(".ts")) {
        const result = await executeScript({
          scriptPath: fullPath,
          inputData: currentData,
        });
        results.push(result);
        currentData = result;
      } else if (item.includes("parallel")) {
        const parallelResults = await handleParallelExecution(
          path,
          item,
          currentData,
        );
        results.push(...parallelResults);
        currentData = {
          data: parallelResults.reduce(
            (acc, result) => ({ ...acc, ...result.data }),
            {},
          ),
        };
      }
    }

    return results;
  } catch (error) {
    state.failedStep = path;
    throw error;
  }
}

async function handleParallelExecution(
  path: string,
  item: string,
  currentData: any,
): Promise<any[]> {
  const parallelTasks = [];
  const parallelFiles = await readDirectory(path);

  for (const scriptFile of parallelFiles.sort()) {
    if (scriptFile.endsWith(".ts")) {
      const scriptPath = `${path}/${item}/${scriptFile}`;
      parallelTasks.push(
        executeScript({
          scriptPath,
          inputData: currentData,
        }),
      );
    }
  }

  return parallelTasks.length > 0 ? Promise.all(parallelTasks) : [];
}
