export interface TaskFunction {
  (): Promise<{ task: string; data: Record<string, any> }>;
}

export interface TaskConfig {
  retries: number;
}

export interface TaskDefinition {
  task: TaskFunction;
  config?: TaskConfig;
}
