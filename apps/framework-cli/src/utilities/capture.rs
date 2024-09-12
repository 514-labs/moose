//! # Capture Utility
//!
//! This module leverages moose to instrument moose. It includes a macro to easily capture data anywhere in the codebase.
//!
use crate::cli::settings::Settings;
use crate::utilities::constants::{CLI_VERSION, CONTEXT, CTX_SESSION_ID};
use chrono::Utc;
use lazy_static::lazy_static;
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

// Create a lazy static instance of the client
lazy_static! {
    pub static ref CLIENT: reqwest::Client = reqwest::Client::new();
}

use chrono::DateTime;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum ActivityType {
    #[serde(rename = "blockInitCommand")]
    BlockInitCommand,
    #[serde(rename = "buildCommand")]
    BuildCommand,
    #[serde(rename = "bumpVersionCommand")]
    PlanCommand,
    #[serde(rename = "planCommand")]
    BumpVersionCommand,
    #[serde(rename = "cleanCommand")]
    CleanCommand,
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
    #[serde(rename = "generateMigrations")]
    GenerateMigrations,
}

#[derive(Debug, Clone, Serialize)]
pub struct MooseActivity {
    pub id: Uuid,
    pub project: String,
    #[serde(rename = "activityType")]
    pub activity_type: ActivityType,
    #[serde(rename = "sequenceId")]
    pub sequence_id: String,
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "cliVersion")]
    pub cli_version: String,
    #[serde(rename = "isMooseDeveloper")]
    pub is_moose_developer: bool,
    #[serde(rename = "machineId")]
    pub machine_id: String,

    pub ip: Option<String>,
}

pub fn capture_usage(
    activity_type: ActivityType,
    project_name: Option<String>,
    settings: &Settings,
) -> Option<tokio::task::JoinHandle<()>> {
    // Ignore our deployments & internal testing
    if settings.telemetry.enabled {
        let mut event = MooseActivity {
            id: Uuid::new_v4(),
            project: project_name.unwrap_or("N/A".to_string()),
            activity_type,
            sequence_id: CONTEXT.get(CTX_SESSION_ID).unwrap().clone(),
            timestamp: Utc::now(),
            cli_version: CLI_VERSION.to_string(),
            is_moose_developer: settings.telemetry.is_moose_developer,
            machine_id: settings.telemetry.machine_id.clone(),
            ip: None,
        };

        Some(tokio::task::spawn(async move {
            match CLIENT
                .get("https://api64.ipify.org?format=text")
                .timeout(Duration::from_secs(2))
                .send()
                .await
            {
                Ok(response) => {
                    event.ip = Some(response.text().await.unwrap());
                }
                Err(e) => {
                    log::warn!("Failed to get IP address for telemetry: {:?}", e);
                }
            }
            let event = json!(event);

            // Sending this data can fail for a variety of reasons, so we don't want to
            // block user & no need to handle the result
            // The API version is pinned on purpose to avoid breaking changes. We
            // can deliberately update this when the schema changes.
            let request = CLIENT
                .post("https://moosefood.514.dev/ingest/MooseActivity/0.4")
                .json(&event)
                .timeout(Duration::from_secs(2));

            let _ = request.send().await;
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
