import {
  log as logger,
  proxyActivities,
  workflowInfo,
  continueAsNew,
  sleep,
} from "@temporalio/workflow";
import { Duration } from "@temporalio/common";
import { Task, Workflow } from "../dmv2";
import { WorkflowState } from "./types";
import { mooseJsonEncode } from "./serialization";

interface ContinueAsNewParams {
  currentWorkflow: string;
  currentTask: string;
}

const { getDmv2Workflow } = proxyActivities({
  startToCloseTimeout: "1 minutes",
  retry: {
    maximumAttempts: 1,
  },
});

export async function ScriptWorkflow(
  params: string | ContinueAsNewParams,
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
  const path = typeof params === "string" ? params : params.currentWorkflow;
  let currentData = inputData?.data || inputData || {};

  logger.info(
    `Starting workflow for ${path} with data: ${JSON.stringify(currentData)}`,
  );

  try {
    currentData = JSON.parse(mooseJsonEncode(currentData));
    const workflow = await getDmv2Workflow(path);
    const task =
      typeof params === "string" ?
        workflow.config.startingTask
      : params.currentTask;
    const result = await handleDmv2Task(workflow, task, currentData);
    results.push(...result);

    return results;
  } catch (error) {
    state.failedStep = path;
    throw error;
  }
}

async function handleDmv2Task(
  workflow: Workflow,
  task: Task<any, any>,
  inputData: any,
): Promise<any[]> {
  const taskTimeout = (task.config.timeout || "1h") as Duration;
  const taskRetries = task.config.retries ?? 3;
  logger.info(
    `Handling task ${task.name} with timeout ${taskTimeout} and retries ${taskRetries}`,
  );

  const { executeDmv2Task } = proxyActivities({
    startToCloseTimeout: taskTimeout,
    heartbeatTimeout: "10s",
    retry: {
      maximumAttempts: taskRetries,
    },
  });

  const monitorTask = async () => {
    logger.info(`Monitor task starting for ${task.name}`);
    for (let historyLimitChecks = 0; ; historyLimitChecks++) {
      const info = workflowInfo();

      // TODO: remove historyLimitChecks >= 10. This is just to test the continue as new functionality
      if (
        info.historyLength >= 800 ||
        info.historySize >= 1048576 ||
        historyLimitChecks >= 10
      ) {
        logger.info(
          `History limits approaching after ${historyLimitChecks} checks`,
        );
        logger.info(
          `Events: ${info.historyLength} | Size: ${info.historySize}`,
        );

        return await continueAsNew({
          currentWorkflow: workflow.name,
          currentTask: task,
        });
      }

      await sleep(100);
    }
  };

  const result = await Promise.race([
    executeDmv2Task(workflow, task, inputData).then((taskResult) => ({
      type: "task_completed",
      data: taskResult,
    })),
    monitorTask().then((continueResult) => ({
      type: "continue_as_new",
      data: continueResult,
    })),
  ]);

  if (result.type === "continue_as_new") {
    logger.info(`Workflow continuing as new due to history limits`);
    return result.data;
  }

  const results = [result.data];
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
    logger.info(`Extract task ${task.name} has more data, restarting chain...`);

    // Recursively call the extract task again to get the next batch
    const nextBatchResults = await handleDmv2Task(workflow, task, null);
    results.push(...nextBatchResults);
  }

  return results;
}
