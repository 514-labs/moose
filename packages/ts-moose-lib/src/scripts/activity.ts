import { log as logger } from "@temporalio/activity";
import * as fs from "fs";
import { Task, Workflow } from "../dmv2";
import { getWorkflows, getTaskForWorkflow } from "../dmv2/internal";
import { WorkflowTaskResult } from "./types";

export interface ScriptExecutionInput {
  scriptPath: string;
  inputData?: any;
}

export const activities = {
  async executeScript(
    input: ScriptExecutionInput,
  ): Promise<WorkflowTaskResult> {
    try {
      const { scriptPath, inputData } = input;

      logger.info(`Task received input: ${JSON.stringify(inputData)}`);

      const processedInput = (inputData || {})?.data || {};
      const scriptModule = await require(scriptPath);
      const execResult = await scriptModule.default();
      const result = await execResult.task(processedInput);

      return result;
    } catch (error) {
      const errorData = {
        error: "Script execution failed",
        details: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined,
      };
      const errorMsg = JSON.stringify(errorData);
      logger.error(errorMsg);
      throw new Error(errorMsg);
    }
  },

  async hasDmv2Workflow(name: string): Promise<boolean> {
    try {
      const workflows = await getWorkflows();
      const hasWorkflow = workflows.has(name);
      logger.info(`<DMV2WF> Found workflow:: ${hasWorkflow}`);
      return hasWorkflow;
    } catch (error) {
      logger.error(
        `<DMV2WF> Failed to check if workflow ${name} exists: ${error}`,
      );
      return false;
    }
  },

  async getDmv2Workflow(name: string): Promise<Workflow> {
    try {
      logger.info(`<DMV2WF> Getting workflow ${name}`);

      const workflows = await getWorkflows();

      if (workflows.has(name)) {
        logger.info(`<DMV2WF> Workflow ${name} found`);
        return workflows.get(name)!;
      } else {
        const errorData = {
          error: "Workflow not found",
          details: `Workflow ${name} not found`,
          stack: undefined,
        };
        const errorMsg = JSON.stringify(errorData);
        logger.error(errorMsg);
        throw new Error(errorMsg);
      }
    } catch (error) {
      const errorData = {
        error: "Failed to get workflow",
        details: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined,
      };
      const errorMsg = JSON.stringify(errorData);
      logger.error(errorMsg);
      throw new Error(errorMsg);
    }
  },

  async executeDmv2Task(
    workflow: Workflow,
    task: Task<any, any>,
    inputData: any,
  ): Promise<any[]> {
    try {
      logger.info(
        `<DMV2WF> Task ${task.name} received input: ${JSON.stringify(inputData)}`,
      );

      // Data between temporal workflow & activities are serialized so we
      // have to get it again to access the user's run function
      const fullTask = await getTaskForWorkflow(workflow.name, task.name);

      return await fullTask.config.run(
        JSON.parse(JSON.stringify(inputData), jsonDateReviver),
      );
    } catch (error) {
      const errorData = {
        error: "Task execution failed",
        details: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined,
      };
      const errorMsg = JSON.stringify(errorData);
      logger.error(errorMsg);
      throw new Error(errorMsg);
    }
  },

  async readDirectory(dirPath: string): Promise<string[]> {
    try {
      const files = fs.readdirSync(dirPath);
      return files;
    } catch (error) {
      const errorMsg = `Failed to read directory ${dirPath}: ${error}`;
      logger.error(errorMsg);
      throw new Error(errorMsg);
    }
  },

  async getActivityRetry(filePath: string): Promise<number> {
    try {
      const scriptModule = await require(filePath);
      const execResult = await scriptModule.default();
      const retriesConfig = execResult?.config?.retries;
      const retries = typeof retriesConfig === "number" ? retriesConfig : 3;
      logger.info(`Using retries in ${filePath}: ${retries}`);
      return retries;
    } catch (error) {
      const errorMsg = `Failed to get task retry for ${filePath}: ${error}`;
      logger.error(errorMsg);
      throw new Error(errorMsg);
    }
  },
};

// Helper function to create activity for a specific script
export function createActivityForScript(scriptName: string) {
  return {
    [scriptName]: activities.executeScript,
  };
}

function jsonDateReviver(key: string, value: unknown): unknown {
  const iso8601Format =
    /^([\+-]?\d{4}(?!\d{2}\b))((-?)((0[1-9]|1[0-2])(\3([12]\d|0[1-9]|3[01]))?|W([0-4]\d|5[0-2])(-?[1-7])?|(00[1-9]|0[1-9]\d|[12]\d{2}|3([0-5]\d|6[1-6])))([T\s]((([01]\d|2[0-3])((:?)[0-5]\d)?|24\:?00)([\.,]\d+(?!:))?)?(\17[0-5]\d([\.,]\d+)?)?([zZ]|([\+-])([01]\d|2[0-3]):?([0-5]\d)?)?)?)$/;

  if (typeof value === "string" && iso8601Format.test(value)) {
    return new Date(value);
  }

  return value;
}
