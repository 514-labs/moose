use itertools::sorted;
use log::{info, warn};
use regex::{Captures, Regex};
use std::{fs, path::Path};

use crate::{
    infrastructure::stream::redpanda::{fetch_topics, RedpandaConfig},
    project::Project,
    utilities::constants::{PY_FLOW_FILE, TS_FLOW_FILE},
};

use super::model::{Flow, FlowError};

const MIGRATION_REGEX: &str = r"^([a-zA-Z0-9_]+)_migrate__([0-9_]+)__(([a-zA-Z0-9_]+)__)?([0-9_]+)";

/**
 * This function gets the flows as defined by the user for the
 * current version of the moose application.
 */
pub async fn get_all_current_flows(project: &Project) -> Result<Vec<Flow>, FlowError> {
    let flows_path = project.flows_dir();
    get_all_flows(&project.redpanda_config, &flows_path).await
}

/**
 * This function should be pointed at the root of the 'flows' directory.
 * Whether this is an old version of the moose app flows (inside the version directory)
 * or the current version of the application.
 *
 * It will assume that all the first children of the directory are the data models
 * sources and the the second children are the data model targets.
 *
 * This also retrieves the migration flows.
 *
 * TODO - handle historical versions. For now this only collects the latest/current version
 * of flows using the latest available topics for each data model.
 */
async fn get_all_flows(config: &RedpandaConfig, path: &Path) -> Result<Vec<Flow>, FlowError> {
    let topics = fetch_topics(config).await?;

    // This should not fail since the regex is hardcoded
    let migration_regex = Regex::new(MIGRATION_REGEX).unwrap();

    let mut flows = vec![];

    for source in fs::read_dir(path)? {
        let source = source?;

        // We check if the file is a migration flow
        if source.metadata()?.is_file() && !source.file_name().to_str().unwrap().ends_with("sql") {
            let potential_migration_file_name = &source.file_name().to_string_lossy().to_string();
            let Some(caps) = migration_regex.captures(potential_migration_file_name) else {
                // This is a file but not a migration flow, so we can skip it
                // and continue to the next iteration
                continue;
            };

            let flow = build_migration_flow(caps, &source.path());
            flows.push(flow);

            // In this case we are currently parsing a migration flow
            // As such we can skip the following steps which are specific
            // to the data model flows
            continue;
        }

        let source_data_model = source.file_name().to_string_lossy().to_string();
        let source_topic = match get_latest_topic(&topics, &source_data_model) {
            Some(topic) => topic,
            None => {
                warn!(
                    "No source topic found in Kafka for data model {}",
                    source_data_model
                );
                continue;
            }
        };

        let target = fs::read_dir(source.path())?;

        for target in target {
            let target = target?;
            let target_data_model = target.file_name().to_string_lossy().to_string();
            let target_topic = match get_latest_topic(&topics, &target_data_model) {
                Some(topic) => topic,
                None => {
                    warn!(
                        "No target topic found in Kafka for data model {}",
                        target_data_model
                    );
                    continue;
                }
            };

            for flow_file in fs::read_dir(target.path())? {
                let flow_file = flow_file?;
                let file_name = flow_file.file_name().to_string_lossy().to_string();

                if flow_file.metadata()?.is_file()
                    && (file_name.starts_with(TS_FLOW_FILE) || file_name.starts_with(PY_FLOW_FILE))
                {
                    let flow = Flow {
                        source_topic: source_topic.clone(),
                        target_topic: target_topic.clone(),
                        executable: flow_file.path().to_path_buf(),
                    };
                    flows.push(flow);

                    // There can only be one flow file per target
                    continue;
                }
            }
        }
    }

    Ok(flows)
}

/**
 * This function retrieves the latest topic for a given data model
 * This should be eventually retired for proper DCM management for flows.
 */
fn get_latest_topic(topics: &[String], data_model: &str) -> Option<String> {
    // Ths algorithm is not super efficient. We probably should have a
    // good way to retrieve the topics for a given data model from the state
    sorted(
        topics
            .iter()
            .filter(|&topic| topic.starts_with(data_model))
            .collect::<Vec<&String>>(),
    )
    .last()
    .map(|topic| topic.to_string())
}

fn build_migration_flow(caps: Captures, executable: &Path) -> Flow {
    info!("Flows Data Captures for migrations {:?}", caps);
    let source_model = caps.get(1).unwrap().as_str();
    let target_model = caps.get(4).map(|m| m.as_str()).unwrap_or(source_model);
    let source_version = caps.get(2).unwrap().as_str();
    let target_version = caps.get(5).unwrap().as_str();

    let source_table = format!("{}_{}", source_model, source_version);
    let target_table = format!("{}_{}", target_model, target_version);

    Flow {
        source_topic: format!("{}_{}_input", source_table, target_table),
        target_topic: format!("{}_{}_output", source_table, target_table),
        executable: executable.to_path_buf(),
    }
}
