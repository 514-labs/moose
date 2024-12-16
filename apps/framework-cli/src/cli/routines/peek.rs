use crate::cli::display::Message;
use crate::infrastructure::olap::clickhouse::mapper::std_table_to_clickhouse_table;
use crate::infrastructure::olap::clickhouse_alt_client::{
    get_pool, retrieve_infrastructure_map, select_some_as_json,
};
use crate::project::Project;

use super::{RoutineFailure, RoutineSuccess};

use futures::StreamExt;
use log::info;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn peek(
    project: Arc<Project>,
    data_model_name: String,
    limit: u8,
    file: Option<PathBuf>,
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

    // ¯\_(ツ)_/¯
    let version_suffix = project.cur_version().as_suffix();
    let dm_with_version = format!("{}_{}_{}", data_model_name, version_suffix, version_suffix);

    let table = infra
        .tables
        .iter()
        .find_map(|(key, table)| {
            if key.to_lowercase() == dm_with_version.to_lowercase() {
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
            "Error fetching table".to_string(),
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

    if let Some(file_path) = file {
        let mut file = File::create(&file_path).await.map_err(|_| {
            RoutineFailure::error(Message::new(
                "Failed".to_string(),
                "Error creating file".to_string(),
            ))
        })?;

        while let Some(result) = stream.next().await {
            if let Ok(value) = result {
                let json = serde_json::to_string(&value).unwrap();
                file.write_all(format!("{}\n", json).as_bytes())
                    .await
                    .map_err(|_| {
                        RoutineFailure::error(Message::new(
                            "Failed".to_string(),
                            "Error writing to file".to_string(),
                        ))
                    })?;
                success_count += 1;
            } else {
                log::error!("Failed to read row");
            }
        }

        Ok(RoutineSuccess::success(Message::new(
            "Peeked".to_string(),
            format!("{} rows written to {:?}", success_count, file_path),
        )))
    } else {
        while let Some(result) = stream.next().await {
            if let Ok(value) = result {
                let message = serde_json::to_string(&value).unwrap();
                println!("{}", message);
                info!("{}", message);
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
}
