import { log as logger, proxyActivities } from "@temporalio/workflow";
import { Duration } from "@temporalio/common";
import { Task, Workflow } from "../dmv2";
import { WorkflowState } from "./types";
import { mooseJsonEncode } from "./serialization";

const { getActivityRetry, getDmv2Workflow, hasDmv2Workflow, readDirectory } =
  proxyActivities({
    startToCloseTimeout: "1 minutes",
    retry: {
      maximumAttempts: 1,
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
    const isDmv2Workflow = await hasDmv2Workflow(path);
    if (isDmv2Workflow) {
      currentData = JSON.parse(mooseJsonEncode(currentData));
      const workflow = await getDmv2Workflow(path);
      const result = await handleDmv2Task(
        workflow,
        workflow.config.startingTask,
        currentData,
      );
      results.push(...result);
    } else {
      if (path.endsWith(".ts")) {
        currentData = JSON.parse(mooseJsonEncode({ data: currentData }));
        const maximumAttempts = await getActivityRetry(path);

        const { executeScript } = proxyActivities({
          startToCloseTimeout: "10 minutes",
          retry: {
            maximumAttempts,
          },
        });

        const result = await executeScript({
          scriptPath: path,
          inputData: currentData,
        });
        return [result];
      } else {
        // Sequential execution
        currentData = JSON.parse(mooseJsonEncode({ data: currentData }));
        const items = await readDirectory(path);
        for (const item of items.sort()) {
          const fullPath = `${path}/${item}`;

          if (item.endsWith(".ts")) {
            const maximumAttempts = await getActivityRetry(fullPath);
            const { executeScript } = proxyActivities({
              startToCloseTimeout: "10 minutes",
              retry: {
                maximumAttempts,
              },
            });

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
      const maximumAttempts = await getActivityRetry(scriptPath);

      const { executeScript } = proxyActivities({
        startToCloseTimeout: "10 minutes",
        retry: {
          maximumAttempts,
        },
      });
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

async function handleDmv2Task(
  workflow: Workflow,
  task: Task<any, any>,
  inputData: any,
): Promise<any[]> {
  const taskTimeout = (task.config.timeout || "1h") as Duration;
  const taskRetries = task.config.retries ?? 3;
  logger.info(
    `<DMV2WF> Handling task ${task.name} with timeout ${taskTimeout} and retries ${taskRetries}`,
  );

  const { executeDmv2Task } = proxyActivities({
    startToCloseTimeout: taskTimeout,
    retry: {
      maximumAttempts: taskRetries,
    },
  });

  const result = await executeDmv2Task(workflow, task, inputData);
  const results = [result];
  if (!task.config.onComplete?.length) {
    return results;
  }

  for (const childTask of task.config.onComplete) {
    const childResult = await handleDmv2Task(workflow, childTask, result);
    results.push(...childResult);
  }

  // Check if this is an ETL extract task that needs to loop
  // ETL extract tasks end with "_extract" and return BatchResult with hasMore
  if (
    task.name.endsWith("_extract") &&
    result &&
    typeof result === "object" &&
    "hasMore" in result &&
    result.hasMore === true
  ) {
    logger.info(
      `<DMV2WF> Extract task ${task.name} has more data, restarting chain...`,
    );

    // Recursively call the extract task again to get the next batch
    const nextBatchResults = await handleDmv2Task(workflow, task, null);
    results.push(...nextBatchResults);
  }

  return results;
}
