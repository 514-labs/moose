use std::{collections::HashMap, sync::Arc};

use crate::{
    cli::display::{show_table, Message},
    infrastructure::olap::{
        self,
        clickhouse::model::ClickHouseSystemTable,
        clickhouse_alt_client::{get_state, ApplicationState},
    },
    project::Project,
};

use super::{RoutineFailure, RoutineSuccess};

pub async fn list_primitives(
    project: Arc<Project>,
    version: &Option<String>,
    limit: &u16,
) -> Result<RoutineSuccess, RoutineFailure> {
    let target_version = version
        .clone()
        .unwrap_or_else(|| project.cur_version().to_owned());

    let current_state = get_current_state(&project).await?;

    let mut output_table_data = convert_to_table_data(&current_state, &target_version);

    let system_tables = get_system_tables(&project, &target_version).await;

    augment_output_table_data(&mut output_table_data, &system_tables);

    let output_table_array = sort_and_limit_table(output_table_data, limit);

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

fn convert_to_table_data(
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

fn sort_and_limit_table(table_data: HashMap<String, Vec<String>>, limit: &u16) -> Vec<Vec<String>> {
    let mut table_array: Vec<Vec<String>> = table_data
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

fn augment_output_table_data(
    output_table_data: &mut HashMap<String, Vec<String>>,
    system_tables: &HashMap<String, ClickHouseSystemTable>,
) {
    for (key, value) in output_table_data.iter_mut() {
        if let Some(system_table) = system_tables.get(key) {
            if system_table.engine == "MergeTree" {
                value[1].clone_from(&system_table.name);
            } else if system_table.engine == "View" {
                value[2].clone_from(&system_table.name);
            }
        }
    }
}
