//! # Capture Utility
//!
//! This module leverages moose to instrument moose. It includes a macro to easily capture data anywhere in the codebase.
//!
use crate::analytics::PostHogClient;
use crate::cli::settings::Settings;
use crate::utilities::constants::{CONTEXT, CTX_SESSION_ID};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub enum ActivityType {
    #[serde(rename = "blockInitCommand")]
    BlockInitCommand,
    #[serde(rename = "buildCommand")]
    BuildCommand,
    #[serde(rename = "planCommand")]
    PlanCommand,
    #[serde(rename = "cleanCommand")]
    CleanCommand,
    #[serde(rename = "checkCommand")]
    CheckCommand,
    #[serde(rename = "consumptionInitCommand")]
    ConsumptionInitCommand,
    #[serde(rename = "devCommand")]
    DevCommand,
    #[serde(rename = "dataModelCommand")]
    DataModelCommand,
    #[serde(rename = "dockerCommand")]
    DockerCommand,
    #[serde(rename = "funcInitCommand")]
    FuncInitCommand,
    #[serde(rename = "initCommand")]
    InitCommand,
    #[serde(rename = "initTemplateCommand")]
    InitTemplateCommand,
    #[serde(rename = "logsCommand")]
    LogsCommand,
    #[serde(rename = "lsCommand")]
    LsCommand,
    #[serde(rename = "prodCommand")]
    ProdCommand,
    #[serde(rename = "psCommand")]
    PsCommand,
    #[serde(rename = "stopCommand")]
    StopCommand,
    #[serde(rename = "metricsCommand")]
    MetricsCommand,
    #[serde(rename = "importCommand")]
    ImportCommand,
    #[serde(rename = "datamodelInitCommand")]
    DataModelInitCommand,
    #[serde(rename = "generateHashCommand")]
    GenerateHashCommand,
    #[serde(rename = "generateSDKCommand")]
    GenerateSDKCommand,
    #[serde(rename = "peekCommand")]
    PeekCommand,
    #[serde(rename = "workflowCommand")]
    WorkflowCommand,
    #[serde(rename = "workflowInitCommand")]
    WorkflowInitCommand,
    #[serde(rename = "workflowRunCommand")]
    WorkflowRunCommand,
    #[serde(rename = "workflowListCommand")]
    WorkflowListCommand,
    #[serde(rename = "workflowResumeCommand")]
    WorkflowResumeCommand,
    #[serde(rename = "workflowTerminateCommand")]
    WorkflowTerminateCommand,
    #[serde(rename = "workflowPauseCommand")]
    WorkflowPauseCommand,
    #[serde(rename = "workflowUnpauseCommand")]
    WorkflowUnpauseCommand,
    #[serde(rename = "workflowStatusCommand")]
    WorkflowStatusCommand,
    #[serde(rename = "templateListCommand")]
    TemplateListCommand,
}

pub fn capture_usage(
    activity_type: ActivityType,
    project_name: Option<String>,
    settings: &Settings,
    machine_id: String,
) -> Option<tokio::task::JoinHandle<()>> {
    // Ignore our deployments & internal testing
    if settings.telemetry.enabled {
        let posthog = PostHogClient::new(settings, machine_id);
        let sequence_id = CONTEXT.get(CTX_SESSION_ID).unwrap().clone();
        let event_id = Uuid::new_v4();
        let project = project_name.unwrap_or("N/A".to_string());

        // Create properties for the event
        let properties = json!({
            "event_id": event_id.to_string(),
            "command": activity_type,
            "sequence_id": sequence_id,
            "project": project,
        });

        Some(tokio::task::spawn(async move {
            if let Err(e) = posthog
                .capture_cli_usage("moose_cli_command", Some(project), properties)
                .await
            {
                log::warn!("Failed to send telemetry to PostHog: {:?}", e);
            }
        }))
    } else {
        None
    }
}

pub async fn wait_for_usage_capture(handle: Option<tokio::task::JoinHandle<()>>) {
    if let Some(handle) = handle {
        let _ = handle.await;
    }
}
