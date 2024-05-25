use crate::cli::display::Message;
use crate::cli::routines::{crawl_schema, RoutineFailure, RoutineSuccess};
use crate::infrastructure::olap::clickhouse::version_sync::{
    generate_sql_version_syncs, get_all_version_syncs, parse_version,
};
use crate::project::Project;

pub async fn generate_migration(project: &Project) -> Result<RoutineSuccess, RoutineFailure> {
    let previous_version = match project
        .supported_old_versions
        .keys()
        .max_by_key(|v| parse_version(v))
    {
        None => {
            return Ok(RoutineSuccess::info(Message {
                action: "No".to_string(),
                details: "old versions found".to_string(),
            }))
        }
        Some(previous_version) => previous_version,
    };

    let fo_versions = crawl_schema(project, &project.old_versions_sorted())
        .await
        .map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to crawl schema".to_string()),
                err,
            )
        })?;

    let version_syncs = get_all_version_syncs(project, &fo_versions).map_err(|err| {
        RoutineFailure::new(
            Message::new("Failed".to_string(), "to generate migrations".to_string()),
            err,
        )
    })?;

    let new_vs_list = generate_sql_version_syncs(
        &project.clickhouse_config.db_name,
        &fo_versions,
        &version_syncs,
        previous_version,
    );

    let flow_dir = project.flows_dir();
    for vs in new_vs_list {
        let file_path = flow_dir.join(format!(
            "{}_migrate__{}__{}.sql",
            vs.model_name,
            vs.source_version.replace('.', "_"),
            vs.dest_version.replace('.', "_")
        ));
        let file_content = vs.sql_migration_function();
        std::fs::write(&file_path, file_content).map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to write migration file".to_string()),
                err,
            )
        })?;
    }

    Ok(RoutineSuccess::success(Message {
        action: "Generated".to_string(),
        details: "migrations".to_string(),
    }))
}
