//! # Capture Utility
//!
//! This module leverages moose to instrument moose. It includes a macro to easily capture data anywhere in the codebase.
//!
use lazy_static::lazy_static;

// Create a lazy static instance of the client
lazy_static! {
    pub static ref CLIENT: reqwest::Client = reqwest::Client::new();
}

use chrono::{DateTime, Utc};

use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub enum ActivityType {
    #[serde(rename = "aggregationInitCommand")]
    AggregationInitCommand,
    #[serde(rename = "buildCommand")]
    BuildCommand,
    #[serde(rename = "bumpVersionCommand")]
    BumpVersionCommand,
    #[serde(rename = "cleanCommand")]
    CleanCommand,
    #[serde(rename = "consumptionInitCommand")]
    ConsumptionInitCommand,
    #[serde(rename = "devCommand")]
    DevCommand,
    #[serde(rename = "dockerCommand")]
    DockerCommand,
    #[serde(rename = "flowInitCommand")]
    FlowInitCommand,
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

macro_rules! capture {
    ($activity_type:expr, $project_name:expr, $settings:expr) => {
        use crate::utilities::capture::{ActivityType, MooseActivity};
        use crate::utilities::constants;
        use crate::utilities::constants::{CONTEXT, CTX_SESSION_ID};
        use chrono::Utc;
        use reqwest::Client;
        use serde_json::json;
        use std::time::Duration;
        use uuid::Uuid;

        // Ignore our deployments & internal testing
        if $settings.telemetry.enabled {
            let client = Client::new();

            let mut event = MooseActivity {
                id: Uuid::new_v4(),
                project: $project_name,
                activity_type: $activity_type,
                sequence_id: CONTEXT.get(CTX_SESSION_ID).unwrap().clone(),
                timestamp: Utc::now(),
                cli_version: constants::CLI_VERSION.to_string(),
                is_moose_developer: $settings.telemetry.is_moose_developer,
                machine_id: $settings.telemetry.machine_id.clone(),
                ip: None,
            };
            tokio::task::spawn(async move {
                match client
                    .get("https://api64.ipify.org?format=text")
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
                let request = client
                    .post("https://moosefood.514.dev/ingest/MooseActivity/0.4")
                    .json(&event)
                    .timeout(Duration::from_secs(2));

                let _ = request.send().await;
            });
        }
    };
}

pub(crate) use capture;
