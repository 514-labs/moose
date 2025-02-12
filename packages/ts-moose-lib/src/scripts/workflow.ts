import { proxyActivities } from "@temporalio/workflow";
import { WorkflowState } from "./types";
import { ScriptExecutionInput } from "./activity";
import { mooseJsonEncode } from "./serialization";

const { executeActivity } = proxyActivities({
  startToCloseTimeout: "10 minutes",
});

// Add workflow export identifier
export const workflowExports = {
  ScriptWorkflow: class ScriptWorkflow {
    public state: WorkflowState = {
      completedSteps: [],
      currentStep: null,
      failedStep: null,
      scriptPath: null,
      inputData: null,
    };

    public getWorkflowState(): WorkflowState {
      return this.state;
    }

    public async resumeExecution(): Promise<any> {
      this.state.failedStep = null;
      const { scriptPath, inputData } = this.state;
      return await executeActivity(scriptPath, {
        args: [{ scriptPath, inputData }],
        retry: {
          maximumAttempts: 3,
        },
      });
    }

    public async executeActivityWithState(
      activityName: string,
      scriptPath: string,
      inputData?: any,
    ): Promise<any> {
      this.state.currentStep = activityName;

      try {
        const result = await executeActivity(activityName, {
          args: [{ scriptPath, inputData }],
          retry: {
            maximumAttempts: 3,
          },
        });
        this.state.completedSteps.push(activityName);
        return result;
      } catch (error) {
        this.state.failedStep = activityName;
        throw error;
      }
    }

    public async run(path: string, inputData?: any): Promise<any[]> {
      const results: any[] = [];
      let currentData = inputData?.data || inputData || {};

      try {
        // Encode input data
        currentData = JSON.parse(mooseJsonEncode({ data: currentData }));

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
            const parallelResults = await this.handleParallelExecution(
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
        this.state.failedStep = path;
        throw error;
      }
    }

    public async handleParallelExecution(
      path: string,
      item: string,
      currentData: any,
    ): Promise<any[]> {
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

      return parallelTasks.length > 0 ? Promise.all(parallelTasks) : [];
    }
  },
};
