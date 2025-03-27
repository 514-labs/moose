//! Module for listing available resources in the Moose framework.
//!
//! This module provides functionality to list database resources (tables, views)
//! and streaming resources (topics) based on the project configuration.

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use super::{setup_redis_client, RoutineFailure, RoutineSuccess};
use crate::framework::core::infrastructure_map::InfrastructureMap;
use crate::{
    cli::display::{show_table, Message},
    infrastructure::{
        olap::{self, clickhouse::model::ClickHouseSystemTable},
        stream::kafka::{self},
    },
    project::Project,
};

/// Lists database resources for a given project and version.
///
/// Retrieves and displays information about API endpoints, tables, and views
/// that are available in the project's current state.
///
/// # Arguments
///
/// * `project` - The project to list database resources for
/// * `version` - Optional specific version to list resources for; defaults to current version
/// * `limit` - Maximum number of resources to display
///
/// # Returns
///
/// * `Result<RoutineSuccess, RoutineFailure>` - Success or failure of the operation
pub async fn list_db(
    project: Arc<Project>,
    version: &Option<String>,
    limit: &u16,
) -> Result<RoutineSuccess, RoutineFailure> {
    let target_version = version
        .clone()
        .unwrap_or_else(|| project.cur_version().to_string());

    let redis_client = setup_redis_client(project.clone()).await.map_err(|e| {
        RoutineFailure::error(Message {
            action: "Prod".to_string(),
            details: format!("Failed to setup redis client: {:?}", e),
        })
    })?;

    let infra = InfrastructureMap::load_from_redis(&redis_client)
        .await
        // temporarily have some duplicate code with get_current_state
        .map_err(|e| {
            RoutineFailure::new(
                Message {
                    action: "Failed".to_string(),
                    details: "Error retrieving current state".to_string(),
                },
                e,
            )
        })?
        .ok_or_else(|| {
            RoutineFailure::error(Message::new(
                "Failed".to_string(),
                "No Moose state found".to_string(),
            ))
        })?;

    let mut output_table = infra
        .api_endpoints
        .values()
        .filter_map(|endpoint| {
            if &endpoint.version == project.cur_version() {
                Some((
                    endpoint.name.clone(),
                    vec![
                        endpoint.path.to_string_lossy().to_string(),
                        String::new(),
                        String::new(),
                    ],
                ))
            } else {
                None
            }
        })
        .collect();

    add_tables_views(&project, &target_version, &mut output_table).await;

    let output_table_array = sort_and_limit(output_table, limit);

    show_table(
        vec![
            "Data Model".to_string(),
            "Ingestion Point".to_string(),
            "Table".to_string(),
            "View".to_string(),
        ],
        output_table_array,
    );

    Ok(RoutineSuccess::success(Message::new(
        "".to_string(),
        "".to_string(),
    )))
}

/// Lists streaming resources (topics) for a given project.
///
/// Retrieves and displays information about Redpanda topics that are
/// available in the project.
///
/// # Arguments
///
/// * `project` - The project to list streaming resources for
/// * `limit` - Maximum number of resources to display
///
/// # Returns
///
/// * `Result<RoutineSuccess, RoutineFailure>` - Success or failure of the operation
pub async fn list_streaming(
    project: Arc<Project>,
    limit: &u16,
) -> Result<RoutineSuccess, RoutineFailure> {
    let topics = get_topics(&project).await;

    let grouped_topics = group_topics_by_prefix(topics);

    let flattened_topics = format_topics(&project, grouped_topics, limit);

    show_table(
        vec!["Data Model".to_string(), "Topic".to_string()],
        flattened_topics,
    );

    Ok(RoutineSuccess::success(Message::new(
        "".to_string(),
        "".to_string(),
    )))
}

/// Sorts a table by the first column and limits the number of rows.
///
/// # Arguments
///
/// * `output_table` - HashMap with keys as first column and values as remaining columns
/// * `limit` - Maximum number of rows to include in output
///
/// # Returns
///
/// * `Vec<Vec<String>>` - Sorted and limited table data
fn sort_and_limit(output_table: HashMap<String, Vec<String>>, limit: &u16) -> Vec<Vec<String>> {
    let mut table_array: Vec<Vec<String>> = output_table
        .into_iter()
        .map(|(key, mut values)| {
            let mut array = vec![key];
            array.append(&mut values);
            array
        })
        .collect();

    table_array.sort_by(|a, b| a[0].cmp(&b[0]));

    if table_array.len() > (*limit as usize) {
        table_array.truncate(*limit as usize);
    }

    table_array
}

/// Retrieves system tables from ClickHouse for a given version.
///
/// # Arguments
///
/// * `project` - The project configuration
/// * `target_version` - The version to get tables for
///
/// # Returns
///
/// * `HashMap<String, ClickHouseSystemTable>` - Map of table names to table information
async fn get_system_tables(
    project: &Project,
    target_version: &str,
) -> HashMap<String, ClickHouseSystemTable> {
    let configured_client = olap::clickhouse::create_client(project.clickhouse_config.clone());

    olap::clickhouse::fetch_tables_with_version(
        &configured_client,
        &format!("%{}", target_version.replace('.', "_")),
    )
    .await
    .unwrap()
    .into_iter()
    .map(|table| (remove_suffix(&table.name), table))
    .collect::<HashMap<_, _>>()
}

/// Removes version suffix from table names.
///
/// # Arguments
///
/// * `table_name` - The table name with suffix
///
/// # Returns
///
/// * `String` - Table name without version suffix
fn remove_suffix(table_name: &str) -> String {
    let parts: Vec<&str> = table_name.split('_').collect();
    if parts.len() > 2 {
        parts[..parts.len() - 2].join("_")
    } else {
        table_name.to_string()
    }
}

/// Adds table and view information to the output table.
///
/// # Arguments
///
/// * `project` - The project configuration
/// * `target_version` - The version to get tables and views for
/// * `output_table` - HashMap to update with table and view information
async fn add_tables_views(
    project: &Project,
    target_version: &str,
    output_table: &mut HashMap<String, Vec<String>>,
) {
    let version_filter = if project.features.data_model_v2 {
        ""
    } else {
        target_version
    };
    let mut system_tables = get_system_tables(project, version_filter).await;

    fn update_metadata(system_table: ClickHouseSystemTable, metadata: &mut [String]) {
        match system_table.engine.as_str() {
            "MergeTree" => {
                if let Some(v) = metadata.get_mut(1) {
                    *v = system_table.name;
                }
            }
            "View" => {
                if let Some(v) = metadata.get_mut(2) {
                    *v = system_table.name;
                }
            }
            _ => {}
        }
    }

    for (data_model, metadata) in output_table.iter_mut() {
        if let Some(system_table) = system_tables.remove(data_model) {
            update_metadata(system_table, metadata);
        }
    }

    // handle system_tables that are not in output_table (i.e. tables with no ingestion endpoint)
    for (data_model, system_table) in system_tables.into_iter() {
        let mut metadata = vec!["".to_string(), "".to_string(), "".to_string()];
        update_metadata(system_table, &mut metadata);

        output_table.insert(data_model, metadata);
    }
}

/// Retrieves a set of topic names from Redpanda, filtering out internal topics.
///
/// # Arguments
///
/// * `project` - The project configuration
///
/// # Returns
///
/// * `HashSet<String>` - Set of topic names
async fn get_topics(project: &Project) -> HashSet<String> {
    let topic_blacklist = HashSet::<String>::from_iter(vec!["__consumer_offsets".to_string()]);
    HashSet::<String>::from_iter(
        kafka::client::fetch_topics(&project.redpanda_config)
            .await
            .unwrap()
            .into_iter()
            .filter(|topic| !topic_blacklist.contains(&topic.name))
            .map(|topic| topic.get_topic_name_without_namespace()),
    )
}

/// Groups topics by their prefix (the part before the first underscore).
///
/// # Arguments
///
/// * `topics` - Set of topic names to group
///
/// # Returns
///
/// * `HashMap<String, Vec<String>>` - Map of prefixes to lists of topic names
fn group_topics_by_prefix(topics: HashSet<String>) -> HashMap<String, Vec<String>> {
    topics.into_iter().fold(HashMap::new(), |mut group, topic| {
        let mut parts = topic.splitn(2, '_');
        if let Some(key) = parts.next() {
            group.entry(key.to_string()).or_default().push(topic);
        }
        group
    })
}

/// Formats topics for display, grouped by data model.
///
/// # Arguments
///
/// * `project` - The project configuration
/// * `grouped_topics` - Topics grouped by prefix
/// * `limit` - Maximum number of data models to display
///
/// # Returns
///
/// * `Vec<Vec<String>>` - Formatted table data for display
fn format_topics(
    project: &Project,
    grouped_topics: HashMap<String, Vec<String>>,
    limit: &u16,
) -> Vec<Vec<String>> {
    let sorted_limited_data_models = sort_and_limit(grouped_topics, limit);

    sorted_limited_data_models
        .into_iter()
        .filter_map(|inner_array| {
            if inner_array.is_empty() {
                None
            } else {
                let data_model = inner_array.first().cloned().unwrap_or_default();
                let topics = if inner_array.len() > 1 {
                    let mut topics_slice = inner_array[1..].to_vec();
                    topics_slice.sort();

                    let topics_with_prefix: Vec<String> = topics_slice
                        .iter()
                        .map(|topic| project.redpanda_config.prefix_with_namespace(topic))
                        .collect();

                    topics_with_prefix.join("\n")
                } else {
                    String::new()
                };
                Some(vec![data_model, topics])
            }
        })
        .collect()
}
