import * as fs from "fs";
import { WorkflowStepResult } from "./types";

export interface ScriptExecutionInput {
  scriptPath: string;
  inputData?: any;
}

export const activities = {
  async executeScript(
    input: ScriptExecutionInput,
  ): Promise<WorkflowStepResult> {
    try {
      const { scriptPath, inputData } = input;

      console.log(`Activity received input: ${JSON.stringify(inputData)}`);

      // TODO: Handle initial input data & passing data between steps
      const processedInput = (inputData || {})?.data || {};
      const scriptModule = await require(scriptPath);
      const execResult = await scriptModule.default();
      const result = await execResult.task();

      return result;
    } catch (error) {
      const errorData = {
        error: "Script execution failed",
        details: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined,
      };
      throw new Error(JSON.stringify(errorData));
    }
  },

  async readDirectory(dirPath: string): Promise<string[]> {
    try {
      const files = fs.readdirSync(dirPath);
      return files;
    } catch (error) {
      throw new Error(`Failed to read directory ${dirPath}: ${error}`);
    }
  },
};

// Helper function to create activity for a specific script
export function createActivityForScript(scriptName: string) {
  return {
    [scriptName]: activities.executeScript,
  };
}
