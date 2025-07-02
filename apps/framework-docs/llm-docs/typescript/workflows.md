# Workflow Orchestration

Moose workflows enable developers to automate sequences of tasks in a maintainable, reliable way. A workflow is a series of tasks that execute in order, built on top of Temporal for enterprise-grade durability and reliability. Each workflow follows these conventions:

- Workflows live in folders inside the `/app/scripts` subdirectory, with the folder name being the workflow name.
- Tasks are scripts with numerical prefixes (e.g., `1.firstTask.ts`).
- Configuration is managed through `config.toml` files.

This workflow abstraction is powered by Temporal under the hood. You can use the Temporal GUI to monitor your workflow runs as they execute, providing extra debugging capabilities.

## Quickstart

To create a new workflow, run the following command:

```bash
npx moose-cli workflow init ExampleWorkflow --tasks firstTask,secondTask
```

This will create:
- A workflow directory: `/app/scripts/ExampleWorkflow/`
- Task files: `1.firstTask.ts`, `2.secondTask.ts`
- Configuration file: `config.toml`

## Workflow Structure

A typical workflow looks like this:

```typescript
// app/scripts/ExampleWorkflow/1.firstTask.ts
import { TaskFunction, TaskDefinition } from "@514labs/moose-lib";

interface FirstTaskInput {
  name: string;
}

interface FirstTaskOutput {
  name: string;
  greeting: string;
  counter: number;
}

const firstTask: TaskFunction = async (input: FirstTaskInput) => {
  const name = input.name ?? "world";
  const greeting = `hello, ${name}!`;
  return {
    task: "firstTask",
    data: {
      name: name,
      greeting: greeting,
      counter: 1
    } as FirstTaskOutput
  };
};

export default function createTask(): TaskDefinition {
  return {
    task: firstTask,
  };
}
```

## Writing Workflow Tasks

Tasks are defined as asynchronous functions (of type `TaskFunction`) that perform some operations and return an object with two key properties: `task` and `data`.

### Task Function Interface

```typescript
export interface TaskFunction {
  (input?: any): Promise<{ task: string; data: any }>;
}

export interface TaskDefinition {
  task: TaskFunction;
  config?: TaskConfig;
}

export interface TaskConfig {
  retries: number;
}
```

The file must export a function that returns a `TaskDefinition` object. This object is used to register the task with the workflow. Inside this `TaskDefinition` object, you must specify the `task`, which is the function you defined above.

## Data Flow Between Tasks

Tasks communicate through their return values. Each task can return a dictionary containing a `data` key. The contents of this `data` key are automatically passed as input parameters to the next task in the workflow.

- Only values inside the `data` object are passed to the next task.
- Supported data types inside `data` include basic types, containers, and JSON-serializable custom classes.

```typescript
// app/scripts/ExampleWorkflow/1.firstTask.ts
import { TaskFunction, TaskDefinition } from "@514labs/moose-lib";

interface FirstTaskInput {
  name: string;
}

const firstTask: TaskFunction = async (input: FirstTaskInput) => {
  const name = input.name ?? "world";
  const greeting = `hello, ${name}!`;
  return {
    task: "firstTask",
    data: {
      name: name,
      greeting: greeting,
      counter: 1
    }
  };
};

export default function createTask(): TaskDefinition {
  return {
    task: firstTask,
  };
}

// app/scripts/ExampleWorkflow/2.secondTask.ts
import { TaskFunction, TaskDefinition } from "@514labs/moose-lib";

interface SecondTaskInput {
  name: string;
  greeting: string;
  counter: number;
}

const secondTask: TaskFunction = async (input: SecondTaskInput) => {
  const nameLength = input.name.length;
  const expandedGreeting = `${input.greeting} Your name is ${nameLength} characters long`;
  // Example operation: Double the counter value
  const counter = input.counter + 1;
  return {
    task: "secondTask",
    data: {
      name: input.name,
      greeting: expandedGreeting,
      counter: counter,
      nameLength: nameLength
    }
  };
};

export default function createTask(): TaskDefinition {
  return {
    task: secondTask,
  };
}
```

## Workflow Configuration

Configure your workflow using the `config.toml` file in your workflow directory:

```toml
name = "ExampleWorkflow"  # Required. The name of the workflow. Matches the workflow folder name.
timeout = "1h"           # Required. Default: 1 hour. Supports format like "30s", "5m", "2h"
retries = 3             # Required. Default: 3 retries
schedule = ""           # Required. Default: Blank. Cron format for scheduled execution
tasks = ["firstTask", "secondTask"]  # Required. The list of tasks to execute. Matches the task file names.
```

### Configuration Options

- **name**: Must match the workflow folder name
- **timeout**: Maximum execution time for the workflow (e.g., "30s", "5m", "1h", "2d")
- **retries**: Number of times to retry the entire workflow on failure
- **schedule**: Cron expression for scheduled execution (e.g., "0 0 * * *" for daily at midnight)
- **tasks**: Array of task names in execution order

### Scheduled Workflows

For recurring workflows, use cron expressions in the schedule field:

```toml
name = "DailyReport"
timeout = "30m"
retries = 2
schedule = "0 9 * * MON-FRI"  # Every weekday at 9 AM
tasks = ["fetchData", "generateReport", "sendEmail"]
```

Common cron patterns:
- `"0 0 * * *"` - Daily at midnight
- `"0 */6 * * *"` - Every 6 hours
- `"0 9 * * MON-FRI"` - Weekdays at 9 AM
- `"0 0 1 * *"` - First day of every month

## Running Workflows

### CLI Commands

```bash
# Initialize a new workflow
npx moose-cli workflow init MyWorkflow --tasks taskOne,taskTwo

# Run a workflow
npx moose-cli workflow run ExampleWorkflow

# Run with input data
npx moose-cli workflow run ExampleWorkflow --input '{"name": "John"}'

# List all workflows
npx moose-cli workflow list

# List workflows with specific status
npx moose-cli workflow list --status running

# Get workflow status
npx moose-cli workflow status ExampleWorkflow

# Terminate a running workflow
npx moose-cli workflow terminate ExampleWorkflow

# Pause a workflow (if supported)
npx moose-cli workflow pause ExampleWorkflow

# Resume a paused workflow
npx moose-cli workflow unpause ExampleWorkflow
```

### Passing Input to Workflows

When you run a workflow, you can pass input to the workflow by using the `--input` flag:

```bash
npx moose-cli workflow run ExampleWorkflow --input '{"name": "John", "email": "john@example.com"}'
```

The input is passed to the workflow as a JSON string and becomes available to the first task.

## Debugging Workflows

### Temporal Dashboard

While the Temporal dashboard is a helpful tool for debugging, access it at:
```
http://localhost:8080/namespaces/default/workflows
```

The dashboard provides:
- **Timeline View**: Visual representation of workflow execution
- **Event History**: Detailed log of all workflow events
- **Retry Information**: Track retry attempts and failures
- **Task Status**: Real-time status of individual tasks

### CLI Monitoring

Use the Moose CLI to monitor and debug workflows without leaving your terminal:

```bash
# Monitor a specific workflow
npx moose-cli workflow status ExampleWorkflow
```

This will print high-level information about the workflow run:

```txt
Workflow Status: ExampleWorkflow
Run ID: 446eab6e-663d-4913-93fe-f79d6109391f
Status: WORKFLOW_EXECUTION_STATUS_COMPLETED ✅
Execution Time: 66s
```

## Error Detection and Handling

Moose provides multiple layers of error protection, both at the workflow and task level:

### Workflow-Level Retries and Timeouts

Moose automatically catches any runtime errors during workflow execution. Errors are logged for debugging, and the orchestrator will retry failed tasks according to the retry count in `config.toml`.

In your workflow's `config.toml` file, you can configure the following options to control workflow behavior, including timeouts and retries:

```toml
name = "ExampleWorkflow"  # Required. The name of the workflow. Matches the workflow folder name.
timeout = "1h"           # Required. Default: 1 hour
retries = 3             # Required. Default: 3 retries
schedule = ""           # Required. Default: Blank
tasks = ["firstTask", "secondTask"]  # Required. The list of tasks to execute. Matches the task file names.
```

### Task-Level Errors and Retries

To configure a task to have its own retry behavior, you can add a `config` object to the task definition, and specify a number of `retries` within that object:

```typescript
import { TaskFunction, TaskDefinition } from "@514labs/moose-lib";

const riskyTask: TaskFunction = async (input: any) => {
  // This operation might fail
  const result = await callExternalAPI(input);
  
  if (!result.success) {
    throw new Error("External API call failed");
  }
  
  return {
    task: "riskyTask",
    data: result
  };
};

export default function createTask(): TaskDefinition {
  return {
    task: riskyTask,
    config: {
      retries: 5  // This task will retry up to 5 times
    }
  };
}
```

### Advanced Error Handling

```typescript
import { TaskFunction, TaskDefinition } from "@514labs/moose-lib";

interface ProcessingInput {
  userId: string;
  data: any[];
}

const processDataTask: TaskFunction = async (input: ProcessingInput) => {
  try {
    // Validate input
    if (!input.userId || !input.data || input.data.length === 0) {
      throw new Error("Invalid input data");
    }

    // Process data with error handling
    const results = [];
    const errors = [];

    for (const item of input.data) {
      try {
        const processed = await processItem(item);
        results.push(processed);
      } catch (error) {
        errors.push(`Failed to process item ${item.id}: ${error.message}`);
      }
    }

    return {
      task: "processDataTask",
      data: {
        results,
        errors,
        successCount: results.length,
        errorCount: errors.length
      }
    };
  } catch (error) {
    // Log error for debugging
    console.error("Task failed:", error);
    throw error; // Re-throw to trigger retry logic
  }
};

export default function createTask(): TaskDefinition {
  return {
    task: processDataTask,
    config: {
      retries: 3
    }
  };
}
```

### Example: Workflow and Task Retry Interplay

When configuring retries, it's important to understand how workflow-level and task-level retries interact. Consider the following scenario:

- **Workflow Retry Policy**: 2 attempts
```toml
retries = 2
```

- **Task Retry Policy**: 3 attempts
```typescript
export default function createTask(): TaskDefinition {
  return {
    task: firstTask,
    config: {
      retries: 3
    }
  };
}
```

If the execution of the workflow encounters an error, the retry sequence would proceed as follows:

1. **Workflow Attempt 1**
   - **Task Attempt 1**: Task fails
   - **Task Attempt 2**: Task fails
   - **Task Attempt 3**: Task fails
   - Workflow attempt fails after exhausting task retries

2. **Workflow Attempt 2**
   - **Task Attempt 1**: Task fails
   - **Task Attempt 2**: Task fails
   - **Task Attempt 3**: Task fails
   - Workflow attempt fails after exhausting task retries

In this example, the workflow will make a total of 2 attempts, and each task within those attempts will retry up to 3 times before the workflow itself retries.

### Terminating Workflows

To terminate a workflow before it has finished running, use the `workflow terminate` command:

```bash
npx moose-cli workflow terminate ExampleWorkflow
```

You cannot run the same workflow concurrently. Use the `terminate` command to stop the workflow before triggering it again.

## Using Workflows with Ingestion Points

When you need to ingest data from a workflow, you should send it to an ingestion endpoint rather than returning it as workflow output. This ensures proper data ingestion into your Moose infrastructure.

### Example: Workflow with Ingestion

```typescript
import { TaskFunction, TaskDefinition } from "@514labs/moose-lib";

interface DataIngestionTaskInput {
  source: string;
  timestamp: Date;
}

const dataIngestionTask: TaskFunction = async (input: DataIngestionTaskInput) => {
  try {
    // Process your data
    const processedData = {
      id: crypto.randomUUID(),
      source: input.source,
      timestamp: input.timestamp,
      value: Math.random() * 100,
      processedAt: new Date()
    };

    // Send to ingestion endpoint
    const response = await fetch("http://localhost:4000/ingest/analytics", {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify(processedData)
    });

    if (!response.ok) {
      throw new Error(`Failed to ingest data: ${response.status} ${response.statusText}`);
    }

    // Return minimal data for workflow tracking
    return {
      task: "dataIngestionTask",
      data: {
        success: true,
        recordId: processedData.id,
        timestamp: new Date().toISOString()
      }
    };
  } catch (error) {
    console.error("Data ingestion failed:", error);
    throw error; // Will trigger retry logic
  }
};

export default function createTask(): TaskDefinition {
  return {
    task: dataIngestionTask,
    config: {
      retries: 3
    }
  };
}
```

### Best Practices for Workflow Ingestion

1. **Use Dedicated Ingestion Endpoints**:
   - Create specific ingestion endpoints for your workflow data
   - Follow the naming convention `/ingest/<ModelName>`
   - Ensure proper schema validation at the ingestion point

2. **Error Handling**:
   - Always check the response status from ingestion endpoints
   - Implement retries for transient failures
   - Log ingestion failures for debugging

3. **Data Validation**:
   - Validate data before sending to ingestion endpoints
   - Ensure data matches the expected schema
   - Handle validation errors gracefully

4. **Monitoring**:
   - Track ingestion success rates
   - Monitor latency of ingestion operations
   - Set up alerts for ingestion failures

### Example: Complete Workflow with Multiple Ingestion Points

```typescript
import { TaskFunction, TaskDefinition } from "@514labs/moose-lib";

interface DataProcessingTaskInput {
  source: string;
  timestamp: Date;
  userId: string;
}

const processAndIngestTask: TaskFunction = async (input: DataProcessingTaskInput) => {
  try {
    // Process data for different ingestion points
    const analyticsData = {
      id: crypto.randomUUID(),
      source: input.source,
      timestamp: input.timestamp,
      userId: input.userId,
      value: Math.random() * 100,
      processedAt: new Date()
    };

    const metricsData = {
      id: crypto.randomUUID(),
      source: input.source,
      timestamp: input.timestamp,
      metric: "processing_time",
      value: 150,
      userId: input.userId
    };

    // Send to multiple ingestion endpoints in parallel
    const [analyticsResponse, metricsResponse] = await Promise.all([
      fetch("http://localhost:4000/ingest/analytics", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(analyticsData)
      }),
      fetch("http://localhost:4000/ingest/metrics", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(metricsData)
      })
    ]);

    // Check all responses
    if (!analyticsResponse.ok) {
      throw new Error(`Analytics ingestion failed: ${analyticsResponse.status}`);
    }
    if (!metricsResponse.ok) {
      throw new Error(`Metrics ingestion failed: ${metricsResponse.status}`);
    }

    return {
      task: "processAndIngestTask",
      data: {
        success: true,
        analyticsId: analyticsData.id,
        metricsId: metricsData.id,
        timestamp: new Date().toISOString()
      }
    };
  } catch (error) {
    console.error("Multi-ingestion failed:", error);
    throw error;
  }
};

export default function createTask(): TaskDefinition {
  return {
    task: processAndIngestTask,
    config: {
      retries: 3
    }
  };
}
```

## Advanced Workflow Patterns

### Human-in-the-Loop Workflows

```typescript
import { TaskFunction, TaskDefinition } from "@514labs/moose-lib";

interface ApprovalTaskInput {
  requestId: string;
  amount: number;
  description: string;
}

const requestApprovalTask: TaskFunction = async (input: ApprovalTaskInput) => {
  // Send approval request
  await fetch("http://localhost:4000/ingest/approval_requests", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      requestId: input.requestId,
      amount: input.amount,
      description: input.description,
      status: "pending",
      requestedAt: new Date()
    })
  });

  // Wait for approval (this would be handled by Temporal's signal mechanism)
  // In practice, this would use Temporal signals to wait for external approval
  
  return {
    task: "requestApprovalTask",
    data: {
      requestId: input.requestId,
      approved: true, // This would come from the approval system
      approvedAt: new Date().toISOString()
    }
  };
};

export default function createTask(): TaskDefinition {
  return {
    task: requestApprovalTask,
    config: {
      retries: 1 // Don't retry approval requests
    }
  };
}
```

### Long-Running Data Processing

```typescript
import { TaskFunction, TaskDefinition } from "@514labs/moose-lib";

interface BatchProcessingInput {
  batchId: string;
  dataSource: string;
  chunkSize: number;
}

const processBatchTask: TaskFunction = async (input: BatchProcessingInput) => {
  let processedCount = 0;
  let errorCount = 0;
  const errors: string[] = [];

  try {
    // Fetch data in chunks to avoid memory issues
    const totalRecords = await getTotalRecords(input.dataSource);
    const chunks = Math.ceil(totalRecords / input.chunkSize);

    for (let i = 0; i < chunks; i++) {
      try {
        const chunk = await fetchDataChunk(input.dataSource, i, input.chunkSize);
        const processedChunk = await processChunk(chunk);
        
        // Ingest processed chunk
        await fetch("http://localhost:4000/ingest/processed_data", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            batchId: input.batchId,
            chunkIndex: i,
            data: processedChunk,
            processedAt: new Date()
          })
        });
        
        processedCount += processedChunk.length;
        
        // Report progress every 10 chunks
        if (i % 10 === 0) {
          console.log(`Processed ${i + 1}/${chunks} chunks`);
        }
      } catch (error) {
        errorCount++;
        errors.push(`Chunk ${i} failed: ${error.message}`);
      }
    }

    return {
      task: "processBatchTask",
      data: {
        batchId: input.batchId,
        processedCount,
        errorCount,
        errors,
        completedAt: new Date().toISOString()
      }
    };
  } catch (error) {
    console.error("Batch processing failed:", error);
    throw error;
  }
};

export default function createTask(): TaskDefinition {
  return {
    task: processBatchTask,
    config: {
      retries: 2
    }
  };
}
```

## Performance Optimization

### Worker Auto-Tuning

Moose workflows support automatic worker tuning to optimize performance:

- **Automatic Slot Adjustment**: Workers automatically adjust based on CPU and memory usage
- **Memory Management**: Prevents out-of-memory errors with intelligent scaling
- **Performance Optimization**: Maximizes efficiency without manual tuning

### Task Parallelization

```typescript
// Create parallel tasks in separate files
// app/scripts/ParallelWorkflow/1.parallelTask.ts
const parallelTask: TaskFunction = async (input: any) => {
  // This task will run concurrently with others
  const results = await Promise.all([
    processTypeA(input),
    processTypeB(input),
    processTypeC(input)
  ]);

  return {
    task: "parallelTask",
    data: {
      results,
      processedAt: new Date().toISOString()
    }
  };
};
```

### Resource Management

```typescript
const resourceIntensiveTask: TaskFunction = async (input: any) => {
  let resources = null;
  
  try {
    // Acquire resources
    resources = await acquireResources();
    
    // Process with resource limits
    const result = await processWithLimits(input, resources);
    
    return {
      task: "resourceIntensiveTask",
      data: result
    };
  } finally {
    // Always clean up resources
    if (resources) {
      await releaseResources(resources);
    }
  }
};
```

## Testing Workflows

### Unit Testing Tasks

```typescript
// test/workflows/exampleWorkflow.test.ts
import { describe, it, expect } from "@jest/globals";

// Import your task function
import createTask from "../../app/scripts/ExampleWorkflow/1.firstTask";

describe("ExampleWorkflow - firstTask", () => {
  it("should process input correctly", async () => {
    const taskDefinition = createTask();
    const input = { name: "Test User" };
    
    const result = await taskDefinition.task(input);
    
    expect(result.task).toBe("firstTask");
    expect(result.data.name).toBe("Test User");
    expect(result.data.greeting).toBe("hello, Test User!");
    expect(result.data.counter).toBe(1);
  });

  it("should handle missing input gracefully", async () => {
    const taskDefinition = createTask();
    const input = {};
    
    const result = await taskDefinition.task(input);
    
    expect(result.data.name).toBe("world");
    expect(result.data.greeting).toBe("hello, world!");
  });
});
```

### Integration Testing

```typescript
// test/workflows/integration.test.ts
import { execSync } from "child_process";

describe("Workflow Integration Tests", () => {
  it("should complete successfully", () => {
    const output = execSync(
      'npx moose-cli workflow run ExampleWorkflow --input \'{"name": "Test"}\'',
      { encoding: "utf-8" }
    );
    
    expect(output).toContain("Workflow 'ExampleWorkflow' started successfully");
  });
});
```

This comprehensive documentation provides complete coverage of Moose TypeScript workflows, including the latest features, best practices, and real-world examples for building robust, scalable workflow systems. 