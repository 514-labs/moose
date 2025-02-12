import { Context } from "@temporalio/activity";
import * as fs from "fs";
import * as path from "path";
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

      // Always provide data parameter, empty object if no input
      const processedInput = (inputData || {})?.data || {};
      console.log(
        `Processed input_data for task: ${JSON.stringify(processedInput)}`,
      );

      if (!fs.existsSync(scriptPath)) {
        throw new Error(`Script not found: ${scriptPath}`);
      }

      // Import the script module
      const scriptModule = await import(scriptPath);

      // Find the moose task
      const taskFunction = Object.values(scriptModule).find(
        (exp: any) => typeof exp === "function" && exp._isMooseTask,
      );

      if (!taskFunction) {
        throw new Error("No @task() function found in script.");
      }

      // Execute the task with processed input
      const result = await (taskFunction as Function)({ data: processedInput });

      // Validate result
      if (!result || typeof result !== "object") {
        throw new Error(
          "Task must return an object with step and data properties",
        );
      }

      if (!result.step || !("data" in result)) {
        throw new Error("Task result must contain step and data properties");
      }

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
};

// Helper function to create activity for a specific script
export function createActivityForScript(scriptName: string) {
  return {
    [scriptName]: activities.executeScript,
  };
}
