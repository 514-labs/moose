import { IJsonSchemaCollection } from "typia";
import { TypedBase } from "./typedBase";
import { Column } from "../dataModels/dataModelTypes";
import { getMooseInternal } from "./internal";

type TaskHandler<T, R> = (input: T) => Promise<R | void>;

export interface TaskConfig<T, R> {
  run: TaskHandler<T, R>;
  onComplete?: Task<R, any>[];
  timeout?: string;
  retries?: number;
}

export class Task<T, R = any> extends TypedBase<T, TaskConfig<T, R>> {
  constructor(name: string, config: TaskConfig<T, R>);

  /** @internal **/
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

export interface WorkflowConfig {
  startingTask: Task<any, any>;
  retries?: number;
  timeout?: string;
  schedule?: string;
}

export class Workflow {
  constructor(
    readonly name: string,
    readonly config: WorkflowConfig,
  ) {
    getMooseInternal().workflows.set(name, this);
  }
}