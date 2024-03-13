//! # Capture Utility
//!
//! This module leverages moose to intrument moose. It includes a macro to easily capture data anywhere in the codebase.
//!
use chrono::serde::ts_seconds;
use lazy_static::lazy_static;

// create a lazy static instance of the client
lazy_static! {
    pub static ref CLIENT: reqwest::Client = reqwest::Client::new();
}

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
enum ActivityType {
    #[serde(rename = "devCommand")]
    DevCommand,
    #[serde(rename = "initCommand")]
    InitCommand,
    #[serde(rename = "cleanCommand")]
    CleanCommand,
    #[serde(rename = "prodCommand")]
    ProdCommand,
    #[serde(rename = "dockerInitCommand")]
    DockerInitCommand,
    #[serde(rename = "dockerBuildCommand")]
    DockerBuildCommand,
}

#[derive(Debug, Clone, Serialize)]
struct UserActivity {
    id: Uuid,
    project: String,
    #[serde(rename = "activityType")]
    activity_type: ActivityType,
    #[serde(rename = "sequenceId")]
    sequence_id: Uuid,
    #[serde(with = "ts_seconds")]
    timestamp: DateTime<Utc>,
    #[serde(rename = "cliVersion")]
    cli_version: String,
}

async fn capture(ingest_point: &str, event: UserActivity) {
    let client = Client::new();

    // Get the environment variables
    let moose_contributor = std::env::var("MOOSE_CONTRIBUTOR").unwrap_or("unknown".to_string());
    let scheme = std::env::var("SCHEME").unwrap_or("http".to_string());
    let host = std::env::var("HOST").unwrap_or("localhost".to_string());
    let port = std::env::var("PORT").unwrap_or("4000".to_string());
    let path = std::env::var("INGESTION_POINT").unwrap_or("UserActivity".to_string());

    // Format a URL with the scheme, host, and port and the path as variables
    let dev_url = format!("{}://{}:{}/ingest/{}", scheme, host, port, path);
    let prod_url = format!("{}://{}/ingest/{}", scheme, host, path);

    println!("{:?}", dev_url);

    let res = client
        .post(dev_url)
        .json(&json!(event))
        .send()
        .await
        .unwrap();

    println!("{:?}", res);

    // if moose_contributor != "unknown" {
    //     let res = client.post(dev_url).json(&json!(event)).send().await?;
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_capture() {
        let event = UserActivity {
            id: Uuid::new_v4(),
            project: "loose_moose".to_string(),
            activity_type: ActivityType::DevCommand,
            sequence_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            cli_version: "0.1.0".to_string(),
        };

        capture("UserActivity", event).await;
    }
}
