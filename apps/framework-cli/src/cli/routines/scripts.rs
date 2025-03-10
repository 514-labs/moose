use anyhow::Result;
use std::convert::TryFrom;

use crate::cli::display::{show_table, Message};
use crate::cli::routines::{RoutineFailure, RoutineSuccess};
use crate::framework::scripts::Workflow;
use crate::infrastructure::orchestration::temporal::DEFAULT_TEMPORTAL_NAMESPACE;
use crate::infrastructure::orchestration::temporal_client::TemporalClientManager;
use crate::project::Project;
use crate::utilities::constants::{APP_DIR, SCRIPTS_DIR};
use crate::utilities::decode_object::decode_base64_to_json;
use chrono::{DateTime, Utc};
use temporal_sdk_core_protos::temporal::api::common::v1::WorkflowExecution;
use temporal_sdk_core_protos::temporal::api::enums::v1::WorkflowExecutionStatus;
use temporal_sdk_core_protos::temporal::api::workflowservice::v1::{
    DescribeWorkflowExecutionRequest, GetWorkflowExecutionHistoryRequest,
    ListWorkflowExecutionsRequest, SignalWorkflowExecutionRequest,
    TerminateWorkflowExecutionRequest,
};

pub async fn init_workflow(
    project: &Project,
    name: &str,
    tasks: Option<String>,
    task: Option<Vec<String>>,
) -> Result<RoutineSuccess, RoutineFailure> {
    // Convert steps string to vector if present
    let task_vec = if let Some(tasks_str) = tasks {
        tasks_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        task.unwrap_or_default()
    };

    // Initialize the workflow using the existing Workflow::init method
    Workflow::init(project, name, &task_vec).map_err(|e| {
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
            "Workflow '{}' initialized successfully\n\nNext Steps:\n1. cd {}/{}/{}\n2. Edit your workflow tasks\n3. Run with: moose-cli workflow run {}",
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

    let run_id: String = workflow
        .start(&project.temporal_config, input)
        .await
        .map_err(|e| {
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
    project: &Project,
    status: Option<String>,
    limit: u32,
) -> Result<RoutineSuccess, RoutineFailure> {
    let mut table_data = Vec::new();

    let client_manager = TemporalClientManager::new(&project.temporal_config);

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

    let response = client_manager
        .execute(|mut client| async move {
            client
                .list_workflow_executions(request)
                .await
                .map_err(|e| anyhow::Error::msg(e.to_string()))
        })
        .await
        .map_err(|e| {
            RoutineFailure::error(Message {
                action: "Workflow".to_string(),
                details: format!("Could not list workflows: {}\n", e),
            })
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
    project: &Project,
    name: &str,
) -> Result<RoutineSuccess, RoutineFailure> {
    let client_manager = TemporalClientManager::new(&project.temporal_config);

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

    client_manager
        .execute(|mut client| async move {
            client
                .terminate_workflow_execution(request)
                .await
                .map_err(|e| anyhow::Error::msg(e.to_string()))
        })
        .await
        .map_err(|e| {
            let error_message = if e
                .to_string()
                .contains("workflow execution already completed")
            {
                format!("Workflow '{}' has already completed", name)
            } else {
                format!("Could not terminate workflow '{}': {}", name, e)
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
    project: &Project,
    name: &str,
) -> Result<RoutineSuccess, RoutineFailure> {
    let client_manager = TemporalClientManager::new(&project.temporal_config);

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

    client_manager
        .execute(|mut client| async move {
            client
                .signal_workflow_execution(request)
                .await
                .map_err(|e| anyhow::Error::msg(e.to_string()))
        })
        .await
        .map_err(|e| {
            RoutineFailure::error(Message {
                action: "Workflow".to_string(),
                details: format!("Could not pause workflow '{}': {}\n", name, e),
            })
        })?;

    Ok(RoutineSuccess::success(Message {
        action: "Workflow".to_string(),
        details: format!("'{}' paused successfully\n", name),
    }))
}

pub async fn unpause_workflow(
    project: &Project,
    name: &str,
) -> Result<RoutineSuccess, RoutineFailure> {
    let client_manager = TemporalClientManager::new(&project.temporal_config);

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

    client_manager
        .execute(|mut client| async move {
            client
                .signal_workflow_execution(request)
                .await
                .map_err(|e| anyhow::Error::msg(e.to_string()))
        })
        .await
        .map_err(|e| {
            RoutineFailure::error(Message {
                action: "Workflow".to_string(),
                details: format!("Could not unpause workflow '{}': {}\n", name, e),
            })
        })?;

    Ok(RoutineSuccess::success(Message {
        action: "Workflow".to_string(),
        details: format!("'{}' unpaused successfully\n", name),
    }))
}

pub async fn get_workflow_status(
    project: &Project,
    name: &str,
    run_id: Option<String>,
    verbose: bool,
    json: bool,
) -> Result<RoutineSuccess, RoutineFailure> {
    let client_manager = TemporalClientManager::new(&project.temporal_config);

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

        let response = client_manager
            .execute(|mut client| async move {
                client
                    .list_workflow_executions(request)
                    .await
                    .map_err(|e| anyhow::Error::msg(e.to_string()))
            })
            .await
            .map_err(|e| {
                RoutineFailure::error(Message {
                    action: "Workflow".to_string(),
                    details: format!("Could not find workflow '{}': {}\n", name, e),
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

    let response = client_manager
        .execute(|mut client| async move {
            client
                .describe_workflow_execution(request)
                .await
                .map_err(|e| anyhow::Error::msg(e.to_string()))
        })
        .await
        .map_err(|e| {
            RoutineFailure::error(Message {
                action: "Workflow".to_string(),
                details: format!("Could not get status for workflow '{}': {}\n", name, e),
            })
        })?;

    let info = response.into_inner().workflow_execution_info.unwrap();

    let status = WorkflowExecutionStatus::try_from(info.status)
        .map(|s| s.as_str_name().to_string())
        .unwrap_or_else(|_| "UNKNOWN".to_string());

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

    // Create a data structure for JSON output
    let mut status_data = serde_json::json!({
        "workflow_name": name,
        "run_id": execution_id,
        "status": status,
        "status_emoji": status_emoji,
        "execution_time_seconds": execution_time.num_seconds(),
        "start_time": start_time,
    });

    if verbose {
        let history_request = GetWorkflowExecutionHistoryRequest {
            namespace: DEFAULT_TEMPORTAL_NAMESPACE.to_string(),
            execution: Some(WorkflowExecution {
                workflow_id: name.to_string(),
                run_id: execution_id.clone(),
            }),
            ..Default::default()
        };

        let history_response = client_manager
            .execute(|mut client| async move {
                client
                    .get_workflow_execution_history(history_request)
                    .await
                    .map_err(|e| anyhow::Error::msg(e.to_string()))
            })
            .await
            .map_err(|e| {
                RoutineFailure::error(Message {
                    action: "Workflow".to_string(),
                    details: format!("Could not fetch history for workflow '{}': {}\n", name, e),
                })
            })?;

        let mut events = Vec::new();
        if let Some(history) = history_response.into_inner().history {
            for event in history.events {
                if let Ok(event_type) =
                    temporal_sdk_core_protos::temporal::api::enums::v1::EventType::try_from(
                        event.event_type,
                    )
                {
                    let timestamp = event.event_time.map_or(String::from("unknown time"), |ts| {
                        chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                            .map_or("invalid time".to_string(), |dt| dt.to_rfc3339())
                    });

                    let mut event_data = serde_json::json!({
                        "timestamp": timestamp,
                        "type": event_type.as_str_name(),
                    });

                    // Add event attributes
                    if let Some(attrs) = &event.attributes {
                        match attrs {
                            temporal_sdk_core_protos::temporal::api::history::v1::history_event::Attributes::ActivityTaskScheduledEventAttributes(attr) => {
                                if let Some(activity_type) = &attr.activity_type {
                                    event_data["activity"] = serde_json::json!(activity_type.name);
                                }
                            },
                            temporal_sdk_core_protos::temporal::api::history::v1::history_event::Attributes::ActivityTaskCompletedEventAttributes(attr) => {
                                if let Some(result) = &attr.result {
                                    for payload in &result.payloads {
                                        if let Ok(data_str) = String::from_utf8(payload.data.clone()) {
                                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data_str) {
                                                event_data["result"] = json;
                                            } else if let Ok(decoded) = decode_base64_to_json(&data_str) {
                                                event_data["result"] = decoded;
                                            }
                                        }
                                    }
                                }
                            },
                            temporal_sdk_core_protos::temporal::api::history::v1::history_event::Attributes::WorkflowExecutionFailedEventAttributes(attr) => {
                                if let Some(failure) = &attr.failure {
                                    event_data["error"] = serde_json::json!(failure.message);
                                }
                            },
                            temporal_sdk_core_protos::temporal::api::history::v1::history_event::Attributes::WorkflowExecutionCompletedEventAttributes(_) => {
                                event_data["details"] = serde_json::json!("Workflow completed successfully");
                            },
                            _ => {}
                        }
                    }
                    events.push(event_data);
                }
            }
            status_data["events"] = serde_json::json!(events);
        }
    }

    if json {
        Ok(RoutineSuccess::success(Message {
            action: "".to_string(),
            details: serde_json::to_string_pretty(&status_data).unwrap(),
        }))
    } else {
        // Existing text output format
        let mut details = format!(
            "Workflow Status: {}\nRun ID: {}\nStatus: {} {}\nExecution Time: {}s\n",
            name,
            execution_id,
            status,
            status_emoji,
            execution_time.num_seconds()
        );

        if verbose {
            let history_request = GetWorkflowExecutionHistoryRequest {
                namespace: DEFAULT_TEMPORTAL_NAMESPACE.to_string(),
                execution: Some(WorkflowExecution {
                    workflow_id: name.to_string(),
                    run_id: execution_id.clone(),
                }),
                ..Default::default()
            };

            details.push_str(&format!("Request: {:?}\n", history_request));

            let history_response = client_manager
                .execute(|mut client| async move {
                    client
                        .get_workflow_execution_history(history_request)
                        .await
                        .map_err(|e| anyhow::Error::msg(e.to_string()))
                })
                .await
                .map_err(|e| {
                    RoutineFailure::error(Message {
                        action: "Workflow".to_string(),
                        details: format!("Could not fetch history for workflow: {}\n", e),
                    })
                })?;

            if let Some(history) = history_response.into_inner().history {
                details.push_str(&format!("\nFound {} events\n", history.events.len()));
                details.push_str("Event History:\n");

                for event in history.events {
                    let code = event.event_type;
                    if let Ok(event_type) =
                        temporal_sdk_core_protos::temporal::api::enums::v1::EventType::try_from(
                            code,
                        )
                    {
                        let timestamp =
                            event.event_time.map_or(String::from("unknown time"), |ts| {
                                chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                                    .map_or("invalid time".to_string(), |dt| dt.to_rfc3339())
                            });

                        // Format the basic event info with bullet point
                        details.push_str(&format!(
                            "  • [{}] {}",
                            timestamp,
                            event_type.as_str_name()
                        ));

                        // Add relevant details based on event type
                        if let Some(attrs) = &event.attributes {
                            match attrs {
                                temporal_sdk_core_protos::temporal::api::history::v1::history_event::Attributes::ActivityTaskScheduledEventAttributes(attr) => {
                                    if let Some(activity_type) = &attr.activity_type {
                                        details.push_str(&format!("\n    Activity: {}", activity_type.name));
                                    }
                                },
                                temporal_sdk_core_protos::temporal::api::history::v1::history_event::Attributes::ActivityTaskCompletedEventAttributes(attr) => {
                                    if let Some(result) = &attr.result {
                                        details.push_str("\n    Result: ");
                                        for payload in &result.payloads {
                                            match String::from_utf8(payload.data.clone()) {
                                                Ok(data_str) => {
                                                    // Try parsing as JSON first
                                                    match serde_json::from_str::<serde_json::Value>(&data_str) {
                                                        Ok(json) => {
                                                            // Pretty print JSON and indent each line
                                                            let json_str = serde_json::to_string_pretty(&json).unwrap_or_default();
                                                            let indented = json_str
                                                                .lines()
                                                                .map(|line| format!("      {}", line))
                                                                .collect::<Vec<_>>()
                                                                .join("\n");
                                                            details.push_str(&format!("\n{}", indented));
                                                        },
                                                        Err(_) => {
                                                            // If not valid JSON, try base64 decoding
                                                            match decode_base64_to_json(&data_str) {
                                                                Ok(decoded) => {
                                                                    // Pretty print decoded JSON and indent each line
                                                                    let json_str = serde_json::to_string_pretty(&decoded).unwrap_or_default();
                                                                    let indented = json_str
                                                                        .lines()
                                                                        .map(|line| format!("      {}", line))
                                                                        .collect::<Vec<_>>()
                                                                        .join("\n");
                                                                    details.push_str(&format!("\n{}", indented));
                                                                },
                                                                Err(e) => {
                                                                    details.push_str(&format!("Failed to parse payload: {}", e));
                                                                }
                                                            }
                                                        }
                                                    }
                                                },
                                                Err(_) => {
                                                    details.push_str(&format!("Invalid UTF-8 in payload: {:?}", payload));
                                                }
                                            }
                                        }
                                    }
                                },
                                temporal_sdk_core_protos::temporal::api::history::v1::history_event::Attributes::WorkflowExecutionFailedEventAttributes(attr) => {
                                    if let Some(failure) = &attr.failure {
                                        details.push_str(&format!("\n    Error: {}", failure.message));
                                    }
                                },
                                temporal_sdk_core_protos::temporal::api::history::v1::history_event::Attributes::WorkflowExecutionCompletedEventAttributes(_) => {
                                    details.push_str("\n    Workflow completed successfully");
                                },
                                _ => {}
                            }
                        }
                        details.push('\n');
                    }
                }
            } else {
                details.push_str("No history found in response\n");
            }
        } else {
            details.push_str("Verbose flag not set\n");
        }

        Ok(RoutineSuccess::success(Message {
            action: "Workflow".to_string(),
            details,
        }))
    }
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
    async fn test_workflow_init_with_tasks() {
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

        for (i, task) in ["extract", "transform", "load"].iter().enumerate() {
            let file_path = workflow_dir.join(format!("{}.{}.py", i + 1, task));
            assert!(file_path.exists(), "Task file {} should exist", task);

            let content = fs::read_to_string(&file_path).unwrap();
            assert!(content.contains("@task()"));

            let expected_string = format!(r#""task": "{}""#, task);
            assert!(
                content.contains(&expected_string),
                "Content should contain '{}'",
                expected_string
            );
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
