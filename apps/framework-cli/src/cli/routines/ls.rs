use std::sync::Arc;

use crate::{
    cli::display::{show_table, Message},
    infrastructure::olap::clickhouse_alt_client::{get_pool, retrieve_current_state},
    project::Project,
};

use super::{RoutineFailure, RoutineSuccess};

pub async fn list_primitives(
    project: Arc<Project>,
    version: &Option<String>,
    limit: &u16,
) -> Result<RoutineSuccess, RoutineFailure> {
    let mut client = get_pool(&project.clickhouse_config)
        .get_handle()
        .await
        .map_err(|_| {
            RoutineFailure::error(Message::new(
                "Failed".to_string(),
                "Could not get database client".to_string(),
            ))
        })?;

    let current_state = retrieve_current_state(&mut client, &project.clickhouse_config)
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
        })?;

    let target_version = version
        .clone()
        .unwrap_or_else(|| project.cur_version().to_owned());

    let mut data: Vec<Vec<String>> = current_state
        .models
        .iter()
        .find(|&(project_version, _)| project_version == &target_version)
        .map(|tuple| {
            tuple
                .1
                .iter()
                .map(|model| {
                    vec![
                        model.data_model.name.clone(),
                        format!("ingest/{}/{}", model.data_model.name, target_version),
                    ]
                })
                .collect()
        })
        .unwrap_or_else(Vec::new);

    data.sort_by(|a, b| a[0].cmp(&b[0]));

    if data.len() > (*limit as usize) {
        data.truncate(*limit as usize);
    }

    show_table(
        vec!["Data Model".to_string(), "Ingestion Point".to_string()],
        data,
    );

    Ok(RoutineSuccess::success(Message::new(
        "".to_string(),
        "".to_string(),
    )))
}
