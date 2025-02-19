import { log as logger } from "@temporalio/activity";
import * as fs from "fs";
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
