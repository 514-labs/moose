import {
  proxyActivities,
  defineQuery,
  defineSignal,
  WorkflowHandle,
} from "@temporalio/workflow";
import { WorkflowState } from "./types";
import { ScriptExecutionInput } from "./activity";
import { RetryPolicy } from "@temporalio/common";

const { executeActivity } = proxyActivities({
  startToCloseTimeout: "10 minutes",
});

@defineWorkflow()
export class ScriptWorkflow {
  private state: WorkflowState = {
    completedSteps: [],
    currentStep: null,
    failedStep: null,
    scriptPath: null,
    inputData: null,
  };

  @defineQuery
  public getWorkflowState(): WorkflowState {
    return this.state;
  }

  @defineSignal
  public async resumeExecution(): Promise<any> {
    this.state.failedStep = null;
    const { scriptPath, inputData } = this.state;
    return await executeActivity(scriptPath, {
      args: [{ scriptPath, inputData }],
      retryPolicy: new RetryPolicy({
        maximumAttempts: 3,
      }),
    });
  }

  private async executeActivityWithState(
    activityName: string,
    scriptPath: string,
    inputData?: any,
  ): Promise<any> {
    this.state.currentStep = activityName;

    try {
      const result = await executeActivity(activityName, {
        args: [{ scriptPath, inputData }],
        retryPolicy: new RetryPolicy({
          maximumAttempts: 3,
        }),
      });
      this.state.completedSteps.push(activityName);
      return result;
    } catch (error) {
      this.state.failedStep = activityName;
      throw error;
    }
  }

  @defineRun
  public async run(path: string, inputData?: any): Promise<any[]> {
    const results: any[] = [];
    let currentData = inputData?.data || inputData || {};

    if (path.endsWith(".ts") || path.endsWith(".js")) {
      const activityName = `${path.split("/").slice(-2, -1)[0]}/${path.split("/").slice(-1)[0].split(".")[0]}`;
      const result = await this.executeActivityWithState(
        activityName,
        path,
        currentData,
      );
      return [result];
    }

    // Sequential execution
    const items = await executeActivity(path);
    for (const item of items.sort()) {
      const fullPath = `${path}/${item}`;

      if (item.endsWith(".ts") || item.endsWith(".js")) {
        const activityName = `${fullPath.split("/").slice(-2, -1)[0]}/${item.split(".")[0]}`;
        const result = await this.executeActivityWithState(
          activityName,
          fullPath,
          currentData,
        );
        results.push(result);
        currentData = result;
      } else if (item.includes("parallel")) {
        const parallelTasks = [];
        const parallelFiles = await executeActivity(path);

        for (const scriptFile of parallelFiles.sort()) {
          if (scriptFile.endsWith(".ts") || scriptFile.endsWith(".js")) {
            const scriptPath = `${path}/${item}/${scriptFile}`;
            const activityName = `${item}/${scriptFile.split(".")[0]}`;
            parallelTasks.push(
              this.executeActivityWithState(
                activityName,
                scriptPath,
                currentData,
              ),
            );
          }
        }

        if (parallelTasks.length > 0) {
          const parallelResults = await Promise.all(parallelTasks);
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

    return results;
  }
}

// Helper decorators
const defineWorkflow = () => (target: any) => {
  target.name = "ScriptWorkflow";
};

const defineQuery = () => (target: any, propertyKey: string) => {
  // Implementation of defineQuery decorator
};

const defineSignal = () => (target: any, propertyKey: string) => {
  // Implementation of defineSignal decorator
};

const defineRun = () => (target: any, propertyKey: string) => {
  // Implementation of defineRun decorator
};
