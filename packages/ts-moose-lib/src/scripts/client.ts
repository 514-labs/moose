import { Client, Connection } from "@temporalio/client";
import { NativeConnection } from "@temporalio/worker";
import { WorkflowState } from "./types";

export interface WorkflowConfig {
  timeout?: string;
  retries?: number;
  schedule?: string;
}

export class WorkflowClient {
  private client: Client;

  constructor(connection: NativeConnection) {
    this.client = new Client({
      connection: Connection.lazy({
        address: "localhost:7233",
      }),
      namespace: "default",
    });
  }

  async executeWorkflow(
    scriptPath: string,
    inputData?: any,
    config: WorkflowConfig = {},
  ): Promise<string> {
    const workflowId = `typescript-script-${Date.now()}-${Math.random().toString(36).slice(2)}`;

    const handle = await this.client.workflow.start("ScriptWorkflow", {
      taskQueue: "typescript-script-queue",
      workflowId,
      args: [scriptPath, inputData],
      workflowRunTimeout: "1 minute",
      retry: {
        maximumAttempts: config.retries || 3,
      },
      cronSchedule: config.schedule,
    });

    return handle.workflowId;
  }

  async getWorkflowState(workflowId: string): Promise<WorkflowState> {
    const handle = await this.client.workflow.getHandle(workflowId);
    return await handle.query("getWorkflowState");
  }
}
