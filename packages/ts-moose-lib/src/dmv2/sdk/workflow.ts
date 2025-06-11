import { IJsonSchemaCollection } from "typia";
import { TypedBase } from "../typedBase";
import { Column } from "../../dataModels/dataModelTypes";
import { getMooseInternal } from "../internal";

/**
 * A function type that handles task execution.
 *
 * @template T - The input type for the task handler (defaults to null for no input)
 * @template R - The return type for the task handler (defaults to null for no output)
 * @param input - The input data to be processed by the task
 * @returns A promise that resolves to the processed result or void
 */
type TaskHandler<T, R> =
  T extends null ? () => Promise<R extends null ? void : R>
  : (input: T) => Promise<R extends null ? void : R>;

/**
 * Configuration options for defining a task within a workflow.
 *
 * @template T - The input type for the task (defaults to null for no input)
 * @template R - The return type for the task (defaults to null for no output)
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
 * @template T - The input type that this task expects (defaults to null for no input)
 * @template R - The return type that this task produces (defaults to null for no output)
 *
 * @example
 * ```typescript
 * // No input, no output
 * const task1 = new Task("task1", {
 *   run: async () => {
 *     console.log("No input/output");
 *   }
 * });
 *
 * // No input, but has output
 * const task2 = new Task<null, OutputType>("task2", {
 *   run: async () => {
 *     return someOutput;
 *   }
 * });
 *
 * // Has input, no output
 * const task3 = new Task<InputType, null>("task3", {
 *   run: async (input: InputType) => {
 *     // process input but return nothing
 *   }
 * });
 *
 * // Has both input and output
 * const task4 = new Task<InputType, OutputType>("task4", {
 *   run: async (input: InputType) => {
 *     return process(input);
 *   }
 * });
 * ```
 */
export class Task<T = null, R = null> extends TypedBase<T, TaskConfig<T, R>> {
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
  startingTask: Task<any, any> | Task<null, void>;

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
