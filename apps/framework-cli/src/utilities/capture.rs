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
    pub sequence_id: Uuid,
    #[serde(with = "ts_seconds")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "cliVersion")]
    pub cli_version: String,
}

macro_rules! capture {
    ($activity_type:expr, $sequence_id:expr, $project_name:expr) => {
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

        // Get the environment variables
        // let moose_contributor = std::env::var("MOOSE_CONTRIBUTOR").unwrap_or("unknown".to_string());
        let scheme = std::env::var("SCHEME").unwrap_or("http".to_string());
        let host = std::env::var("HOST").unwrap_or("localhost".to_string());
        let port = std::env::var("PORT").unwrap_or("4000".to_string());
        let path = std::env::var("INGESTION_POINT").unwrap_or("UserActivity".to_string());

        // Format a URL with the scheme, host, and port and the path as variables
        #[allow(unused)]
        let dev_url = format!("{}://{}:{}/ingest/{}", scheme, host, port, path);
        // let prod_url = format!("{}://{}/ingest/{}", scheme, host, path);

        // let client = Client::new();
        // let res = client
        //     .post(&dev_url)
        //     .json(event)
        //     .send()
        //     .await
        //     .unwrap();

        // println!("Sent to {}. Event: {}", dev_url, event);
    };
}

pub(crate) use capture;
