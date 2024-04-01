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
}

macro_rules! capture {
    ($activity_type:expr, $sequence_id:expr, $project_name:expr) => {
        use crate::project::PROJECT;
        use crate::utilities::capture::{ActivityType, MooseActivity};
        use crate::utilities::constants;
        use chrono::Utc;
        use reqwest::Client;
        use serde_json::json;
        use std::env;
        use std::time::Duration;
        use uuid::Uuid;

        // Ignore our deployments & internal testing
        let moose_internal = env::var("MOOSE_INTERNAL");
        if moose_internal.is_err() {
            let event = json!(MooseActivity {
                id: Uuid::new_v4(),
                project: $project_name,
                activity_type: $activity_type,
                sequence_id: $sequence_id,
                timestamp: Utc::now(),
                cli_version: constants::CLI_VERSION.to_string(),
            });
            let remote_url = {
                let guard = PROJECT.lock().unwrap();
                guard.instrumentation_config.url().clone()
            };

            let instrumentation_url = format!("{}/ingest/MooseActivity", remote_url);
            println!("Sending to: {}. Event: {}", &instrumentation_url, &event);

            // Sending this data can fail for a variety of reasons, so we don't want to
            // block user & no need to handle the result
            let client = Client::new();
            let request = client
                .post(&instrumentation_url)
                .json(&event)
                .timeout(Duration::from_secs(2));
            let _ = request.send().await;
        }
    };
}

pub(crate) use capture;
