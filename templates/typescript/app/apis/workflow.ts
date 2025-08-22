import { Api } from "@514labs/moose-lib";

interface WorkflowResponse {
  workflowId: string;
  status: string;
}

const triggerApi = new Api<{}, WorkflowResponse>(
<<<<<<< HEAD
  "start-workflow",
=======
  "start-generator-workflow",
>>>>>>> 7938e271 (APis)
  async (_, { client }) => {
    // Trigger the workflow with input parameters
    const workflowExecution = await client.workflow.execute("generator", {});

    return {
      workflowId: workflowExecution.body,
      status: "started",
    };
  },
);

const terminateAPI = new Api<{}, WorkflowResponse>(
<<<<<<< HEAD
  "terminate-workflow",
=======
  "terminate-generator-workflow",
>>>>>>> 7938e271 (APis)
  async (_, { client }) => {
    // Terminate the workflow by name
    const workflowExecution = await client.workflow.terminate("generator");

    return {
      workflowId: workflowExecution.body,
      status: "terminated",
    };
  },
);

export { triggerApi, terminateAPI };
