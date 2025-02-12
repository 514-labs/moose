export interface TaskFunction {
  (): Promise<{ step: string; data: Record<string, any> }>;
}

export interface TaskConfig {
  retries: number;
}

export interface TaskDefinition {
  task: TaskFunction;
  config?: TaskConfig;
}
