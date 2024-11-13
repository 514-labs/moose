use crate::cli::display::Message;
use crate::infrastructure::olap::clickhouse::mapper::std_table_to_clickhouse_table;
use crate::infrastructure::olap::clickhouse_alt_client::{
    get_pool, retrieve_infrastructure_map, select_some_as_json,
};
use crate::project::Project;

use super::{RoutineFailure, RoutineSuccess};

use futures::StreamExt;
use std::sync::Arc;

pub async fn peek(
    project: Arc<Project>,
    data_model_name: String,
    limit: u8,
) -> Result<RoutineSuccess, RoutineFailure> {
    let pool = get_pool(&project.clickhouse_config);
    let mut client = pool.get_handle().await.map_err(|_| {
        RoutineFailure::error(Message::new(
            "Failed".to_string(),
            "Error connecting to storage".to_string(),
        ))
    })?;

    let infra = retrieve_infrastructure_map(&mut client, &project.clickhouse_config)
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
                "No state found".to_string(),
            ))
        })?;

    let table = infra
        .tables
        .iter()
        .find_map(|(key, table)| {
            if key
                .to_lowercase()
                .starts_with(&data_model_name.to_lowercase())
            {
                Some(table)
            } else {
                None
            }
        })
        .ok_or_else(|| {
            RoutineFailure::error(Message::new(
                "Failed".to_string(),
                "No matching table found".to_string(),
            ))
        })?;

    let table = std_table_to_clickhouse_table(table).map_err(|_| {
        RoutineFailure::error(Message::new(
            "Failed".to_string(),
            "Error fetching ClickHouse table".to_string(),
        ))
    })?;

    let mut stream = select_some_as_json(
        &project.clickhouse_config.db_name,
        &table,
        &mut client,
        limit as i64,
    )
    .await
    .map_err(|_| {
        RoutineFailure::error(Message::new(
            "Failed".to_string(),
            "Error selecting data".to_string(),
        ))
    })?;

    let mut success_count = 0;

    while let Some(result) = stream.next().await {
        if let Ok(value) = result {
            let json = serde_json::to_string(&value).unwrap();
            println!("{}", json);
            success_count += 1;
        } else {
            log::error!("Failed to read row");
        }
    }

    // Just a newline for output cleanliness
    println!();

    Ok(RoutineSuccess::success(Message::new(
        "Peeked".to_string(),
        format!("{} rows", success_count),
    )))
}
