use anyhow::Result;
use std::convert::TryFrom;

use crate::cli::display::{show_table, Message};
use crate::cli::routines::{RoutineFailure, RoutineSuccess};
use crate::framework::scripts::Workflow;
use crate::infrastructure::orchestration::temporal::{
    get_temporal_client, DEFAULT_TEMPORTAL_NAMESPACE,
};
use crate::project::Project;
use crate::utilities::constants::{APP_DIR, SCRIPTS_DIR};
use chrono::{DateTime, Utc};
use temporal_sdk_core_protos::temporal::api::enums::v1::WorkflowExecutionStatus;
use temporal_sdk_core_protos::temporal::api::workflowservice::v1::{
    DescribeWorkflowExecutionRequest, ListWorkflowExecutionsRequest,
    SignalWorkflowExecutionRequest, TerminateWorkflowExecutionRequest,
};

pub async fn init_workflow(
    project: &Project,
    name: &str,
    steps: Option<String>,
    step: Option<Vec<String>>,
) -> Result<RoutineSuccess, RoutineFailure> {
    // Convert steps string to vector if present
    let step_vec = if let Some(steps_str) = steps {
        steps_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        step.unwrap_or_default()
    };

    // Initialize the workflow using the existing Workflow::init method
    Workflow::init(project, name, &step_vec).map_err(|e| {
        RoutineFailure::new(
            Message {
                action: "Workflow Init Failed".to_string(),
                details: format!("Could not initialize workflow '{}': {}", project.name(), e),
            },
            e,
        )
    })?;

    // Return success with helpful next steps
    Ok(RoutineSuccess::success(Message {
        action: "Created".to_string(),
        details: format!(
            "Workflow '{}' initialized successfully\n\nNext Steps:\n1. cd {}/{}/{}\n2. Edit your workflow steps\n3. Run with: moose-cli workflow run {}",
            name, APP_DIR, SCRIPTS_DIR, name, name
        ),
    }))
}

pub async fn run_workflow(
    project: &Project,
    name: &str,
    input: Option<String>,
) -> Result<RoutineSuccess, RoutineFailure> {
    let workflow_dir = project.scripts_dir().join(name);

    // Check if workflow directory exists
    if !workflow_dir.exists() {
        return Err(RoutineFailure::error(Message {
            action: "Workflow".to_string(),
            details: format!(
                "'{}' not found in directory {}\n",
                name,
                workflow_dir.display()
            ),
        }));
    }

    let workflow: Workflow = Workflow::from_dir(workflow_dir.clone()).map_err(|e| {
        RoutineFailure::new(
            Message {
                action: "Workflow".to_string(),
                details: format!(
                    "Could not create workflow '{}' from directory {}: {}\n",
                    name,
                    workflow_dir.display(),
                    e
                ),
            },
            e,
        )
    })?;

    let run_id: String = workflow.start(input).await.map_err(|e| {
        RoutineFailure::new(
            Message {
                action: "Workflow".to_string(),
                details: format!("Could not start workflow '{}': {}\n", name, e),
            },
            e,
        )
    })?;

    // Check if run_id is empty or invalid
    if run_id.is_empty() {
        return Err(RoutineFailure::new(
            Message {
                action: "Workflow".to_string(),
                details: format!("'{}' failed to start: Invalid run ID\n", name),
            },
            anyhow::anyhow!("Invalid run ID"),
        ));
    }

    let dashboard_url = format!(
        "http://localhost:8080/namespaces/{}/workflows/{}/{}/history",
        DEFAULT_TEMPORTAL_NAMESPACE, name, run_id
    );

    Ok(RoutineSuccess::success(Message {
        action: "Workflow".to_string(),
        details: format!(
            "'{}' started successfully.\nView it in the Temporal dashboard: {}\n",
            name, dashboard_url
        ),
    }))
}

pub async fn list_workflows(
    _project: &Project,
    status: Option<String>,
    limit: u32,
) -> Result<RoutineSuccess, RoutineFailure> {
    let mut table_data = Vec::new();

    let mut client = match get_temporal_client().await {
        Ok(client) => client,
        Err(e) => {
            return Err(RoutineFailure::new(
                Message {
                    action: "Workflow".to_string(),
                    details: "Could not connect to Temporal.\n".to_string(),
                },
                e,
            ));
        }
    };

    // Convert status string to Temporal status enum
    let status_filter = if let Some(status_str) = status {
        match status_str.to_lowercase().as_str() {
            "running" => Some(WorkflowExecutionStatus::Running),
            "completed" => Some(WorkflowExecutionStatus::Completed),
            "failed" => Some(WorkflowExecutionStatus::Failed),
            _ => None,
        }
    } else {
        None
    };

    // Build query string for Temporal
    let query = if let Some(status) = status_filter {
        format!("ExecutionStatus = '{}'", status.as_str_name())
    } else {
        "".to_string()
    };

    // List workflows from Temporal
    let request = ListWorkflowExecutionsRequest {
        namespace: DEFAULT_TEMPORTAL_NAMESPACE.to_string(),
        page_size: limit as i32,
        query,
        ..Default::default()
    };

    let response = client
        .list_workflow_executions(request)
        .await
        .map_err(|e| {
            RoutineFailure::new(
                Message {
                    action: "Workflow".to_string(),
                    details: format!("Could not list workflows: {}\n", e),
                },
                e,
            )
        })?;

    // Convert workflow executions to table data
    let response_inner = response.into_inner();
    for execution in response_inner.executions {
        if let Some(_workflow_type) = execution.r#type {
            let status = WorkflowExecutionStatus::try_from(execution.status)
                .map_or("UNKNOWN".to_string(), |s| s.as_str_name().to_string());

            if let Some(execution_info) = execution.execution {
                table_data.push(vec![
                    execution_info.workflow_id,
                    execution_info.run_id,
                    status,
                    execution.start_time.map_or("-".to_string(), |t| {
                        chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32)
                            .map_or("-".to_string(), |dt| dt.to_string())
                    }),
                ]);
            }
        }
    }

    // Show table with workflow information
    show_table(
        vec![
            "Workflow Name".to_string(),
            "Run ID".to_string(),
            "Status".to_string(),
            "Started At".to_string(),
        ],
        table_data,
    );

    Ok(RoutineSuccess::success(Message::new(
        "Workflows".to_string(),
        "Listed\n".to_string(),
    )))
}

pub async fn terminate_workflow(
    _project: &Project,
    name: &str,
) -> Result<RoutineSuccess, RoutineFailure> {
    let mut client = get_temporal_client().await.map_err(|_e| {
        RoutineFailure::error(Message {
            action: "Workflow".to_string(),
            details: "Could not connect to Temporal.\n".to_string(),
        })
    })?;

    let request = TerminateWorkflowExecutionRequest {
        namespace: DEFAULT_TEMPORTAL_NAMESPACE.to_string(),
        workflow_execution: Some(
            temporal_sdk_core_protos::temporal::api::common::v1::WorkflowExecution {
                workflow_id: name.to_string(),
                run_id: "".to_string(),
            },
        ),
        reason: "Terminated by user request".to_string(),
        ..Default::default()
    };

    client
        .terminate_workflow_execution(request)
        .await
        .map_err(|e| {
            let error_message = match e.message() {
                "workflow execution already completed" => {
                    format!("Workflow '{}' has already completed", name)
                }
                _ => format!("Could not terminate workflow '{}': {}", name, e.message()),
            };

            RoutineFailure::error(Message {
                action: "Workflow".to_string(),
                details: format!("{}\n", error_message),
            })
        })?;

    Ok(RoutineSuccess::success(Message {
        action: "Workflow".to_string(),
        details: format!("'{}' terminated successfully\n", name),
    }))
}

pub async fn pause_workflow(
    _project: &Project,
    name: &str,
) -> Result<RoutineSuccess, RoutineFailure> {
    let mut client = get_temporal_client().await.map_err(|e| {
        RoutineFailure::new(
            Message {
                action: "Workflow".to_string(),
                details: "Could not connect to Temporal.\n".to_string(),
            },
            e,
        )
    })?;

    let request = SignalWorkflowExecutionRequest {
        namespace: DEFAULT_TEMPORTAL_NAMESPACE.to_string(),
        workflow_execution: Some(
            temporal_sdk_core_protos::temporal::api::common::v1::WorkflowExecution {
                workflow_id: name.to_string(),
                run_id: "".to_string(),
            },
        ),
        signal_name: "pause".to_string(),
        input: None,
        ..Default::default()
    };

    client
        .signal_workflow_execution(request)
        .await
        .map_err(|e| {
            RoutineFailure::new(
                Message {
                    action: "Workflow".to_string(),
                    details: format!("Could not pause workflow '{}': {}\n", name, e),
                },
                e,
            )
        })?;

    Ok(RoutineSuccess::success(Message {
        action: "Workflow".to_string(),
        details: format!("'{}' paused successfully\n", name),
    }))
}

pub async fn unpause_workflow(
    _project: &Project,
    name: &str,
) -> Result<RoutineSuccess, RoutineFailure> {
    let mut client = get_temporal_client().await.map_err(|e| {
        RoutineFailure::new(
            Message {
                action: "Workflow".to_string(),
                details: "Could not connect to Temporal.\n".to_string(),
            },
            e,
        )
    })?;

    let request = SignalWorkflowExecutionRequest {
        namespace: DEFAULT_TEMPORTAL_NAMESPACE.to_string(),
        workflow_execution: Some(
            temporal_sdk_core_protos::temporal::api::common::v1::WorkflowExecution {
                workflow_id: name.to_string(),
                run_id: "".to_string(),
            },
        ),
        signal_name: "unpause".to_string(),
        input: None,
        ..Default::default()
    };

    client
        .signal_workflow_execution(request)
        .await
        .map_err(|e| {
            RoutineFailure::new(
                Message {
                    action: "Workflow".to_string(),
                    details: format!("Could not unpause workflow '{}': {}\n", name, e),
                },
                e,
            )
        })?;

    Ok(RoutineSuccess::success(Message {
        action: "Workflow".to_string(),
        details: format!("'{}' unpaused successfully\n", name),
    }))
}

pub async fn get_workflow_status(
    _project: &Project,
    name: &str,
    run_id: Option<String>,
) -> Result<RoutineSuccess, RoutineFailure> {
    let mut client = get_temporal_client().await.map_err(|_| {
        RoutineFailure::error(Message {
            action: "Workflow".to_string(),
            details: "Could not connect to Temporal.\n".to_string(),
        })
    })?;

    // If no run_id provided, get the most recent one
    let execution_id = if let Some(id) = run_id {
        id
    } else {
        // List workflows to get most recent
        let request = ListWorkflowExecutionsRequest {
            namespace: DEFAULT_TEMPORTAL_NAMESPACE.to_string(),
            page_size: 1,
            query: format!("WorkflowId = '{}'", name),
            ..Default::default()
        };

        let response = client
            .list_workflow_executions(request)
            .await
            .map_err(|e| {
                RoutineFailure::error(Message {
                    action: "Workflow".to_string(),
                    details: format!("Could not find workflow '{}': {}\n", name, e.message()),
                })
            })?;

        let executions = response.into_inner().executions;
        if executions.is_empty() {
            return Err(RoutineFailure::error(Message {
                action: "Workflow".to_string(),
                details: format!("No executions found for workflow '{}'\n", name),
            }));
        }

        executions[0].execution.as_ref().unwrap().run_id.clone()
    };

    // Get workflow details
    let request = DescribeWorkflowExecutionRequest {
        namespace: DEFAULT_TEMPORTAL_NAMESPACE.to_string(),
        execution: Some(
            temporal_sdk_core_protos::temporal::api::common::v1::WorkflowExecution {
                workflow_id: name.to_string(),
                run_id: execution_id.clone(),
            },
        ),
    };

    let response = client
        .describe_workflow_execution(request)
        .await
        .map_err(|e| {
            RoutineFailure::error(Message {
                action: "Workflow".to_string(),
                details: format!(
                    "Could not get status for workflow '{}': {}\n",
                    name,
                    e.message()
                ),
            })
        })?;

    let info = response.into_inner().workflow_execution_info.unwrap();

    // Format the output
    let status_emoji = match WorkflowExecutionStatus::try_from(info.status) {
        Ok(status) => match status {
            WorkflowExecutionStatus::Running => "⏳",
            WorkflowExecutionStatus::Completed => "✅",
            WorkflowExecutionStatus::Failed => "❌",
            _ => "❓",
        },
        Err(_) => "❓",
    };

    let start_time = DateTime::<Utc>::from_timestamp(
        info.start_time.as_ref().unwrap().seconds,
        info.start_time.as_ref().unwrap().nanos as u32,
    )
    .unwrap();

    let execution_time = Utc::now().signed_duration_since(start_time);

    let details = format!(
        "Status: {}\nRun ID: {}\nStatus: {} {}\nExecution Time: {}s\n",
        name,
        execution_id,
        info.status().as_str_name(),
        status_emoji,
        execution_time.num_seconds()
    );

    Ok(RoutineSuccess::success(Message {
        action: "Workflow".to_string(),
        details,
    }))
}

#[cfg(test)]
mod tests {
    use crate::framework::languages::SupportedLanguages;
    use crate::project::Project;

    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> Project {
        let temp_dir = TempDir::new().unwrap();
        let project = Project::new(
            temp_dir.path(),
            "project-name".to_string(),
            SupportedLanguages::Python,
        );
        project
    }

    #[tokio::test]
    async fn test_workflow_init_basic() {
        let project = setup();

        let result = init_workflow(&project, "daily-etl", None, None)
            .await
            .unwrap();

        assert!(result.message.details.contains("daily-etl"));

        let workflow_dir = project.app_dir().join(SCRIPTS_DIR).join("daily-etl");

        assert!(
            workflow_dir.exists(),
            "Workflow directory should be created in app/scripts"
        );

        let config_path = workflow_dir.join("config.toml");
        assert!(config_path.exists(), "config.toml should be created");
    }

    #[tokio::test]
    async fn test_workflow_init_with_steps() {
        let project = setup();

        let result = init_workflow(
            &project,
            "daily-etl",
            Some("extract,transform,load".to_string()),
            None,
        )
        .await
        .unwrap();

        assert!(result.message.details.contains("daily-etl"));

        let workflow_dir = project.app_dir().join(SCRIPTS_DIR).join("daily-etl");

        for (i, step) in ["extract", "transform", "load"].iter().enumerate() {
            let file_path = workflow_dir.join(format!("{}.{}.py", i + 1, step));
            assert!(file_path.exists(), "Step file {} should exist", step);

            let content = fs::read_to_string(&file_path).unwrap();
            assert!(content.contains("@task"));
        }
    }

    #[tokio::test]
    async fn test_workflow_init_failure() {
        let project = setup();

        // Create a file where the workflow directory should be to cause a failure
        fs::create_dir_all(project.app_dir().join(SCRIPTS_DIR)).unwrap();
        fs::write(project.app_dir().join(SCRIPTS_DIR).join("daily-etl"), "").unwrap();

        let result = init_workflow(&project, "daily-etl", None, None).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message.action, "Workflow Init Failed");
    }

    // Test is ignored because it requires temporal as a dependency
    #[ignore]
    #[tokio::test]
    async fn test_run_workflow() {
        let project = setup();

        const WORKFLOW_NAME: &str = "daily-etl";

        // First initialize a workflow
        let _ = init_workflow(
            &project,
            WORKFLOW_NAME,
            Some("extract,transform,load".to_string()),
            None,
        )
        .await
        .unwrap();

        // Verify workflow exists
        let workflow_dir = project.app_dir().join(SCRIPTS_DIR).join(WORKFLOW_NAME);
        assert!(workflow_dir.exists(), "Workflow directory should exist");

        // Run the workflow
        let result = run_workflow(&project, WORKFLOW_NAME, None).await;
        println!("Result: {:?}", result);
        assert!(result.is_ok(), "Workflow should run successfully");

        let success = result.unwrap();
        assert!(success.message.details.contains("started successfully"));
    }
}
