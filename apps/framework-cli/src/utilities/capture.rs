//! # Capture Utility
//!
//! This module leverages moose to instrument moose. It includes a macro to easily capture data anywhere in the codebase.
//!
use chrono::serde::ts_seconds;
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
    #[serde(rename = "buildCommand")]
    BuildCommand,
    #[serde(rename = "bumpVersionCommand")]
    BumpVersionCommand,
    #[serde(rename = "cleanCommand")]
    CleanCommand,
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
    #[serde(rename = "prodCommand")]
    ProdCommand,
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
    #[serde(with = "ts_seconds")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "cliVersion")]
    pub cli_version: String,
    #[serde(rename = "isMooseDeveloper")]
    pub is_moose_developer: bool,
    #[serde(rename = "machineId")]
    pub machine_id: String,
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
            let event = json!(MooseActivity {
                id: Uuid::new_v4(),
                project: $project_name,
                activity_type: $activity_type,
                sequence_id: CONTEXT.get(CTX_SESSION_ID).unwrap().clone(),
                timestamp: Utc::now(),
                cli_version: constants::CLI_VERSION.to_string(),
                is_moose_developer: $settings.telemetry.is_moose_developer,
                machine_id: $settings.telemetry.machine_id.clone(),
            });

            // Sending this data can fail for a variety of reasons, so we don't want to
            // block user & no need to handle the result
            let client = Client::new();
            let request = client
                .post("https://moosefood.514.dev/ingest/MooseActivity")
                .json(&event)
                .timeout(Duration::from_secs(2));
            let _ = request.send().await;
        }
    };
}

pub(crate) use capture;
