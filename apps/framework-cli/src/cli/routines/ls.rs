use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    cli::display::{show_table, Message},
    infrastructure::{
        olap::{
            self,
            clickhouse::model::ClickHouseSystemTable,
            clickhouse_alt_client::{get_state, ApplicationState},
        },
        stream::redpanda::{self, RedpandaConfig},
    },
    project::Project,
};

use super::{RoutineFailure, RoutineSuccess};

pub async fn list_db(
    project: Arc<Project>,
    version: &Option<String>,
    limit: &u16,
) -> Result<RoutineSuccess, RoutineFailure> {
    let target_version = version
        .clone()
        .unwrap_or_else(|| project.cur_version().to_owned());

    let current_state = get_current_state(&project).await?;

    let mut output_table = map_models_to_resources(&current_state, &target_version);

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

async fn get_current_state(project: &Project) -> Result<ApplicationState, RoutineFailure> {
    get_state(&project.clickhouse_config)
        .await
        .map_err(|_| {
            RoutineFailure::error(Message::new(
                "Failed".to_string(),
                "Error retrieving current state".to_string(),
            ))
        })?
        .ok_or_else(|| {
            RoutineFailure::error(Message::new(
                "Failed".to_string(),
                "No Moose state found".to_string(),
            ))
        })
}

fn map_models_to_resources(
    current_state: &ApplicationState,
    target_version: &str,
) -> HashMap<String, Vec<String>> {
    current_state
        .models
        .iter()
        .find(|&(project_version, _)| project_version == target_version)
        .map(|tuple| {
            let mut map = HashMap::new();
            for model in &tuple.1 {
                // Map of data model to ingest point, table, and view
                map.insert(
                    model.data_model.name.clone(),
                    vec![
                        format!("ingest/{}/{}", model.data_model.name, target_version),
                        "".to_string(),
                        "".to_string(),
                    ],
                );
            }
            map
        })
        .unwrap_or_else(HashMap::new)
}

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

fn remove_suffix(table_name: &str) -> String {
    let parts: Vec<&str> = table_name.split('_').collect();
    if parts.len() > 2 {
        parts[..parts.len() - 2].join("_")
    } else {
        table_name.to_string()
    }
}

async fn add_tables_views(
    project: &Project,
    target_version: &str,
    output_table: &mut HashMap<String, Vec<String>>,
) {
    let system_tables = get_system_tables(project, target_version).await;

    for (data_model, metadata) in output_table.iter_mut() {
        if let Some(system_table) = system_tables.get(data_model) {
            match system_table.engine.as_str() {
                "MergeTree" => {
                    if let Some(v) = metadata.get_mut(1) {
                        v.clone_from(&system_table.name);
                    }
                }
                "View" => {
                    if let Some(v) = metadata.get_mut(2) {
                        v.clone_from(&system_table.name);
                    }
                }
                _ => {}
            }
        }
    }
}

async fn get_topics(project: &Project) -> HashSet<String> {
    let topic_blacklist = HashSet::<String>::from_iter(vec!["__consumer_offsets".to_string()]);
    HashSet::<String>::from_iter(
        redpanda::fetch_topics(&project.redpanda_config)
            .await
            .unwrap()
            .into_iter()
            .map(|topic| RedpandaConfig::get_topic_without_namespace(&topic))
            .filter(|topic| !topic_blacklist.contains(topic)),
    )
}

fn group_topics_by_prefix(topics: HashSet<String>) -> HashMap<String, Vec<String>> {
    topics.into_iter().fold(HashMap::new(), |mut group, topic| {
        let mut parts = topic.splitn(2, '_');
        if let Some(key) = parts.next() {
            group.entry(key.to_string()).or_default().push(topic);
        }
        group
    })
}

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
                        .map(|topic| project.redpanda_config.get_topic_with_namespace(topic))
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
