import { IJsonSchemaCollection } from "typia";
import { TypedBase } from "../typedBase";
import { Column } from "../../dataModels/dataModelTypes";
import { getMooseInternal } from "../internal";

/**
 * A function type that handles task execution.
 *
 * @template T - The input type for the task handler
 * @template R - The return type for the task handler
 * @param input - The input data to be processed by the task
 * @returns A promise that resolves to the processed result or void
 */
type TaskHandler<T, R> = (input: T) => Promise<R | void>;

/**
 * Configuration options for defining a task within a workflow.
 *
 * @template T - The input type for the task
 * @template R - The return type for the task
 */
export interface TaskConfig<T, R> {
  /** The main function that executes the task logic */
  run: TaskHandler<T, R>;

  /** Optional array of tasks to execute after this task completes successfully */
  onComplete?: Task<R, any>[];

  /** Optional timeout duration for the task execution (e.g., "30s", "5m") */
  timeout?: string;

  /** Optional number of retry attempts if the task fails */
  retries?: number;
}

/**
 * Represents a single task within a workflow system.
 *
 * A Task encapsulates the execution logic, completion handlers, and configuration
 * for a unit of work that can be chained with other tasks in a workflow.
 *
 * @template T - The input type that this task expects
 * @template R - The return type that this task produces (defaults to any)
 *
 * @example
 * ```typescript
 * const processDataTask = new Task("processData", {
 *   run: async (data: RawData) => {
 *     return processData(data);
 *   },
 *   timeout: "30s",
 *   retries: 3
 * });
 * ```
 */
export class Task<T, R = any> extends TypedBase<T, TaskConfig<T, R>> {
  /**
   * Creates a new Task instance.
   *
   * @param name - Unique identifier for the task
   * @param config - Configuration object defining the task behavior
   */
  constructor(name: string, config: TaskConfig<T, R>);

  /**
   * Internal constructor with additional schema and column parameters.
   *
   * @internal
   * @param name - Unique identifier for the task
   * @param config - Configuration object defining the task behavior
   * @param schema - JSON schema collection for type validation
   * @param columns - Column definitions for data structure
   */
  constructor(
    name: string,
    config: TaskConfig<T, R>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
  );

  constructor(
    name: string,
    config: TaskConfig<T, R>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
  ) {
    super(name, config, schema, columns);
  }
}

/**
 * Configuration options for defining a workflow.
 *
 * A workflow orchestrates the execution of multiple tasks in a defined sequence
 * or pattern, with support for scheduling, retries, and timeouts.
 */
export interface WorkflowConfig {
  /** The initial task that begins the workflow execution */
  startingTask: Task<any, any>;

  /** Optional number of retry attempts if the entire workflow fails */
  retries?: number;

  /** Optional timeout duration for the entire workflow execution (e.g., "10m", "1h") */
  timeout?: string;

  /** Optional cron-style schedule string for automated workflow execution */
  schedule?: string;
}

/**
 * Represents a complete workflow composed of interconnected tasks.
 *
 * A Workflow manages the execution flow of multiple tasks, handling scheduling,
 * error recovery, and task orchestration. Once created, workflows are automatically
 * registered with the internal Moose system.
 *
 * @example
 * ```typescript
 * const dataProcessingWorkflow = new Workflow("dataProcessing", {
 *   startingTask: extractDataTask,
 *   schedule: "0 2 * * *", // Run daily at 2 AM
 *   timeout: "1h",
 *   retries: 2
 * });
 * ```
 */
export class Workflow {
  /**
   * Creates a new Workflow instance and registers it with the Moose system.
   *
   * @param name - Unique identifier for the workflow
   * @param config - Configuration object defining the workflow behavior and task orchestration
   */
  constructor(
    readonly name: string,
    readonly config: WorkflowConfig,
  ) {
    getMooseInternal().workflows.set(name, this);
  }
}
