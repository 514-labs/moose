//! # Capture Utility
//!
//! This module leverages moose to instrument moose. It includes a macro to easily capture data anywhere in the codebase.
//!
// use chrono::serde::ts_seconds;
// use lazy_static::lazy_static;

// Create a lazy static instance of the client
lazy_static! {
    pub static ref CLIENT: reqwest::Client = reqwest::Client::new();
}

use chrono::{DateTime, Utc};

use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub enum ActivityType {
    #[serde(rename = "devCommand")]
    DevCommand,
    #[serde(rename = "initCommand")]
    InitCommand,
    #[serde(rename = "cleanCommand")]
    CleanCommand,
    #[serde(rename = "stopCommand")]
    StopCommand,
    #[serde(rename = "prodCommand")]
    ProdCommand,
    #[serde(rename = "dockerInitCommand")]
    DockerInitCommand,
    #[serde(rename = "dockerBuildCommand")]
    DockerBuildCommand,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserActivity {
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
        use crate::utilities::capture::{ActivityType, UserActivity};
        use crate::utilities::constants;
        use chrono::Utc;
        // use reqwest::Client;
        use serde_json::json;
        use uuid::Uuid;

        #[allow(unused)]
        let event = &json!(UserActivity {
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
        #[allow(unused)]
        let prod_url = format!("{}/ingest/UserActivity", remote_url);

        // let client = Client::new();
        // let res = client
        //     .post(&dev_url)
        //     .json(event)
        //     .send()
        //     .await
        //     .unwrap();

        // println!("Sent to {}. Event: {}", prod_url, event);
    };
}

pub(crate) use capture;
