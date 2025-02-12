export interface WorkflowState {
  completedSteps: string[];
  currentStep: string | null;
  failedStep: string | null;
  scriptPath: string | null;
  inputData: any | null;
}

export interface WorkflowStepResult {
  step: string;
  data: any;
}
