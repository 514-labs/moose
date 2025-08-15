import { ConsumptionApi } from "@514labs/moose-lib";

interface WorkflowResponse {
  workflowId: string;
  status: string;
}

const triggerApi = new ConsumptionApi<{}, WorkflowResponse>(
  "start-workflow",
  async (_, { client }) => {
    // Trigger the workflow with input parameters
    const workflowExecution = await client.workflow.execute("workflow", {});

    return {
      workflowId: workflowExecution.body,
      status: "started",
    };
  },
);

const terminateAPI = new ConsumptionApi<{}, WorkflowResponse>(
  "terminate-workflow",
  async (_, { client }) => {
    // Trigger the workflow with input parameters
    const workflowExecution = await client.workflow.terminate("workflow");

    return {
      workflowId: workflowExecution.body,
      status: "started",
    };
  },
);

export { triggerApi, terminateAPI };
