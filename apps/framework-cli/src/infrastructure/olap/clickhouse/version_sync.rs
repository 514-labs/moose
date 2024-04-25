use crate::framework::controller::{FrameworkObject, FrameworkObjectVersions};
use crate::infrastructure::olap::clickhouse::model::{
    ClickHouseColumn, ClickHouseColumnType, ClickHouseTable,
};
use crate::infrastructure::olap::clickhouse::queries::{
    CreateVersionSyncTriggerQuery, InitialLoadQuery,
};
use crate::project::Project;
use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use std::collections::HashMap;
use std::ffi::OsStr;

use super::errors::ClickhouseError;

pub fn parse_version(v: &str) -> Vec<i32> {
    v.split('.')
        .map(|s| s.parse::<i32>().unwrap_or(0))
        .collect::<Vec<i32>>()
}

pub fn version_to_string(v: &[i32]) -> String {
    v.iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join(".")
}

pub fn generate_version_syncs(
    framework_object_versions: &FrameworkObjectVersions,
    version_sync_list: &[VersionSync],
    previous_version: &str,
) -> Vec<VersionSync> {
    let mut previous_models: HashMap<String, FrameworkObject> = framework_object_versions
        .previous_version_models
        .get(previous_version)
        .unwrap()
        .models
        .clone();

    for vs in version_sync_list {
        if vs.source_version == previous_version {
            previous_models.remove(&vs.model_name);
        }
    }

    let mut res = vec![];
    for fo in framework_object_versions.current_models.models.values() {
        if let Some(old_model) = previous_models.get(&fo.data_model.name) {
            if old_model.data_model.columns != fo.data_model.columns {
                res.push(VersionSync {
                    db_name: old_model.table.db_name.clone(),
                    model_name: old_model.data_model.name.clone(),
                    source_version: previous_version.to_string(),
                    source_table: old_model.table.clone(),
                    dest_version: framework_object_versions.current_version.clone(),
                    dest_table: fo.table.clone(),
                    migration_function: VersionSync::generate_migration_function(
                        &old_model.table.columns,
                        &fo.table.columns,
                    ),
                });
            }
        }
    }
    res
}

pub fn get_all_version_syncs(
    project: &Project,
    framework_object_versions: &FrameworkObjectVersions,
) -> anyhow::Result<Vec<VersionSync>> {
    let flows_dir = project.flows_dir();
    let mut version_syncs = vec![];
    for entry in std::fs::read_dir(flows_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            if let Some(captures) =
                VERSION_SYNC_REGEX.captures(entry.file_name().to_string_lossy().as_ref())
            {
                let from_table_name = captures.get(1).unwrap().as_str();
                let to_table_name = captures.get(4).map_or(from_table_name, |m| m.as_str());

                let from_version = captures.get(2).unwrap().as_str().replace('_', ".");
                let to_version = captures.get(5).unwrap().as_str().replace('_', ".");

                let from_version_models = framework_object_versions
                    .previous_version_models
                    .get(&from_version);
                let to_version_models = if to_version == framework_object_versions.current_version {
                    Some(&framework_object_versions.current_models)
                } else {
                    framework_object_versions
                        .previous_version_models
                        .get(&to_version)
                };

                match (from_version_models, to_version_models) {
                    (Some(from_version_models), Some(to_version_models)) => {
                        let from_table = from_version_models.models.get(from_table_name);
                        let to_table = to_version_models.models.get(to_table_name);
                        match (from_table, to_table) {
                            (Some(from_table), Some(to_table)) => {
                                let version_sync = VersionSync {
                                    db_name: from_table.table.db_name.clone(),
                                    model_name: from_table.data_model.name.clone(),
                                    source_version: from_version.clone(),
                                    source_table: from_table.table.clone(),
                                    dest_version: to_version.clone(),
                                    dest_table: to_table.table.clone(),
                                    migration_function: std::fs::read_to_string(&path)?,
                                };
                                version_syncs.push(version_sync);
                            }
                            _ => {
                                return Err(anyhow::anyhow!(
                                    "Failed to find tables in versions {:?}",
                                    path.file_name()
                                        .unwrap_or(OsStr::new("File name not found"))
                                ));
                            }
                        }
                    }
                    _ => {
                        debug!(
                            "Version unavailable for version sync {:?}. from: {:?} to: {:?}",
                            path.file_name(),
                            from_version_models,
                            to_version_models,
                        );
                    }
                }
            };

            debug!("Processing version sync: {:?}.", path);
        }
    }
    Ok(version_syncs)
}

#[derive(Debug, Clone)]
pub struct VersionSync {
    pub db_name: String,
    pub model_name: String,
    pub source_version: String,
    pub source_table: ClickHouseTable,
    pub dest_version: String,
    pub dest_table: ClickHouseTable,
    pub migration_function: String,
}

lazy_static! {
    pub static ref VERSION_SYNC_REGEX: Regex =
        //            source_model_name         source     target_model_name   dest_version
        Regex::new(r"^([a-zA-Z0-9_]+)_migrate__([0-9_]+)__(([a-zA-Z0-9_]+)__)?([0-9_]+).sql$")
            .unwrap();
}

impl VersionSync {
    pub fn write_new_version_sync() {}

    pub fn generate_migration_function(
        old_columns: &[ClickHouseColumn],
        new_columns: &[ClickHouseColumn],
    ) -> String {
        let input = old_columns
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<String>>()
            .join(", ");

        let mut old_column_index = 0;
        let new_columns = new_columns
            .iter()
            .map(|c| {
                if let Some(old_column) = old_columns.get(old_column_index) {
                    if old_column.name == c.name {
                        old_column_index += 1;
                        if old_column.column_type == c.column_type {
                            return c.name.clone();
                        };
                    }
                }
                match &c.column_type {
                    ClickHouseColumnType::String => format!("'{}'", c.name),
                    ClickHouseColumnType::Boolean => "true".to_string(),
                    ClickHouseColumnType::ClickhouseInt(_) => "0".to_string(),
                    ClickHouseColumnType::ClickhouseFloat(_) => "0.0".to_string(),
                    ClickHouseColumnType::Decimal => "0.0".to_string(),
                    ClickHouseColumnType::DateTime => "'2024-02-20T23:14:57.788Z'".to_string(),
                    ClickHouseColumnType::Json => format!("'{{\"{}\": null}}'", c.name),
                    ClickHouseColumnType::Bytes => "0.0".to_string(),
                    ClickHouseColumnType::Enum(data_enum) => {
                        format!("'{}'", data_enum.values[0])
                    }
                    ClickHouseColumnType::Array(_) => "[]".to_string(),
                }
            })
            .collect::<Vec<String>>()
            .join(", ");

        format!("({}) -> ({})", input, new_columns)
    }

    pub fn migration_function_name(&self) -> String {
        format!(
            "{}_migrate__{}__{}",
            self.model_name,
            self.source_version.replace('.', "_"),
            self.dest_version.replace('.', "_"),
        )
    }

    pub fn migration_trigger_name(&self) -> String {
        format!(
            "{}_trigger__{}__{}",
            self.model_name,
            self.source_version.replace('.', "_"),
            self.dest_version.replace('.', "_"),
        )
    }

    pub fn create_function_query(&self) -> String {
        format!(
            "CREATE FUNCTION {} AS {}",
            self.migration_function_name(),
            self.migration_function
        )
    }
    pub fn drop_function_query(&self) -> String {
        format!("DROP FUNCTION IF EXISTS {}", self.migration_function_name())
    }

    pub fn create_trigger_query(self) -> Result<String, ClickhouseError> {
        CreateVersionSyncTriggerQuery::build(self)
    }

    pub fn initial_load_query(self) -> Result<String, ClickhouseError> {
        InitialLoadQuery::build(self)
    }

    pub fn drop_trigger_query(&self) -> String {
        format!(
            "DROP VIEW IF EXISTS {}.{}",
            self.db_name,
            self.migration_trigger_name()
        )
    }
}
