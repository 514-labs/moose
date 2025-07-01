import { log as logger } from "@temporalio/activity";
import * as fs from "fs";
import { Task, Workflow } from "../dmv2";
import { getWorkflows, getTaskForWorkflow } from "../dmv2/internal";
import { jsonDateReviver } from "../utilities/json";
import { WorkflowTaskResult } from "./types";
import { pathToFileURL } from "url";

export interface ScriptExecutionInput {
  scriptPath: string;
  inputData?: any;
}

export const activities = {
  async executeScript(
    input: ScriptExecutionInput,
  ): Promise<WorkflowTaskResult> {
    const { scriptPath, inputData } = input;
    try {
      logger.info(`Task received input: ${JSON.stringify(inputData)}`);

      const processedInput = (inputData || {})?.data || {};
      // Dynamically import the script so both CommonJS and pure-ESM user code work
      const scriptModule = await import(pathToFileURL(scriptPath).href);
      const execResult = await scriptModule.default();
      const result = await execResult.task(processedInput);

      return result;
    } catch (error) {
      const rawDetails = error instanceof Error ? error.message : String(error);
      let hint: string | undefined;
      if (rawDetails.includes("ERR_REQUIRE_ESM")) {
        hint =
          "The script or one of its dependencies is published as an ESM-only module. Moose now loads scripts with dynamic import, but make sure you are using proper export syntax (e.g. `export default`). If you are importing a CommonJS-only file, add `.cjs` extension or convert it.";
      } else if (rawDetails.includes("Cannot find module")) {
        hint = `Could not resolve module. Verify that the path ${scriptPath} exists and that dependencies are installed.`;
      }

      const errorData = {
        error: "Script execution failed",
        details: rawDetails,
        hint,
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

      // Revive any JSON serialized dates in the input data
      const revivedInputData =
        inputData ?
          JSON.parse(JSON.stringify(inputData), jsonDateReviver)
        : inputData;

      return await fullTask.config.run(revivedInputData);
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
      // Use dynamic import here as well for ESM compatibility
      const scriptModule = await import(pathToFileURL(filePath).href);
      const execResult = await scriptModule.default();
      const retriesConfig = execResult?.config?.retries;
      const retries = typeof retriesConfig === "number" ? retriesConfig : 3;
      logger.info(`Using retries in ${filePath}: ${retries}`);
      return retries;
    } catch (error) {
      let hint: string | undefined;
      const details = error instanceof Error ? error.message : String(error);
      if (details.includes("ERR_REQUIRE_ESM")) {
        hint =
          "The task file or its dependencies are ESM-only. Ensure it uses `export default` and that Moose supports dynamic imports for it.";
      } else if (details.includes("Cannot find module")) {
        hint = `Cannot locate file ${filePath}. Confirm the path is correct and the file exists.`;
      }

      const errorMsg = `Failed to get task retry for ${filePath}: ${details}. ${hint ?? ""}`;
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
