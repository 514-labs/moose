export interface TaskFunction {
  (input?: any): Promise<{ task: string; data: any }>;
}

export interface TaskConfig {
  retries: number;
}

export interface TaskDefinition {
  task: TaskFunction;
  config?: TaskConfig;
}
