import { proxyActivities } from "@temporalio/workflow";
import { WorkflowState } from "./types";
import { mooseJsonEncode } from "./serialization";

const { executeScript, readDirectory } = proxyActivities({
  startToCloseTimeout: "10 minutes",
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

  try {
    // Encode input data
    currentData = JSON.parse(mooseJsonEncode({ data: currentData }));

    if (path.endsWith(".ts") || path.endsWith(".js")) {
      const activityName = `${path.split("/").slice(-2, -1)[0]}/${path.split("/").slice(-1)[0].split(".")[0]}`;
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

      if (item.endsWith(".ts") || item.endsWith(".js")) {
        const activityName = `${fullPath.split("/").slice(-2, -1)[0]}/${item.split(".")[0]}`;
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
    if (scriptFile.endsWith(".ts") || scriptFile.endsWith(".js")) {
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
