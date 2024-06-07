use crate::framework::core::code_loader::{FrameworkObject, FrameworkObjectVersions};
use crate::framework::data_model::schema::DataModel;
use crate::infrastructure::olap::clickhouse::model::{
    ClickHouseColumn, ClickHouseColumnType, ClickHouseTable,
};
use crate::infrastructure::olap::clickhouse::queries::create_version_sync_trigger_query;
use crate::project::Project;
use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use super::errors::ClickhouseError;
use super::queries::create_initial_data_load_query;

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

// to be removed when we move to all TS flow version syncs
pub fn generate_sql_version_syncs(
    db_name: &str,
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
            if let Some(old_table) = &old_model.table {
                if let Some(new_table) = &fo.table {
                    if old_model.data_model.columns != fo.data_model.columns {
                        res.push(VersionSync {
                            db_name: db_name.to_string(),
                            model_name: old_model.data_model.name.clone(),
                            source_version: previous_version.to_string(),
                            source_data_model: old_model.data_model.clone(),
                            source_table: old_table.clone(),
                            dest_version: framework_object_versions.current_version.clone(),
                            dest_table: new_table.clone(),
                            sync_type: VersionSyncType::Sql(
                                VersionSync::generate_migration_function(
                                    &old_table.columns,
                                    &new_table.columns,
                                ),
                            ),
                        });
                    }
                }
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

                let sync_type = match captures.get(6).unwrap().as_str() {
                    "sql" => VersionSyncType::Sql(std::fs::read_to_string(&path)?),
                    "ts" => VersionSyncType::Ts(path.clone()),
                    _ => panic!(),
                };

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
                        match (
                            from_version_models.models.get(from_table_name),
                            to_version_models.models.get(to_table_name),
                        ) {
                            (Some(from_table_fo), Some(to_table_fo)) => {
                                if let Some(from_table) = &from_table_fo.table {
                                    if let Some(to_table) = &to_table_fo.table {
                                        let version_sync = VersionSync {
                                            db_name: project.clickhouse_config.db_name.clone(),
                                            model_name: from_table_fo.data_model.name.clone(),
                                            source_version: from_version.clone(),
                                            source_table: from_table.clone(),
                                            source_data_model: from_table_fo.data_model.clone(),
                                            dest_version: to_version.clone(),
                                            dest_table: to_table.clone(),
                                            sync_type,
                                        };
                                        version_syncs.push(version_sync);
                                    }
                                }
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
pub enum VersionSyncType {
    Sql(String),
    Ts(PathBuf),
}

#[derive(Debug, Clone)]
pub struct VersionSync {
    pub db_name: String,
    pub model_name: String,
    pub source_version: String,
    pub source_table: ClickHouseTable,
    pub source_data_model: DataModel,
    pub dest_version: String,
    pub dest_table: ClickHouseTable,
    pub sync_type: VersionSyncType,
}

lazy_static! {
    pub static ref VERSION_SYNC_REGEX: Regex =
        //            source_model_name         source     target_model_name   dest_version
        Regex::new(r"^([a-zA-Z0-9_]+)_migrate__([0-9_]+)__(([a-zA-Z0-9_]+)__)?([0-9_]+).(sql|ts)$")
            .unwrap();
}

impl VersionSync {
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
                    ClickHouseColumnType::Nested(_) => {
                        todo!("Implement the nested type mapper")
                    }
                    ClickHouseColumnType::Enum(data_enum) => {
                        format!("'{}'", data_enum.values[0].name)
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

    pub fn topic_name(&self, suffix: &str) -> String {
        format!(
            "{}_{}_{}",
            self.source_table.name, self.dest_table.name, suffix
        )
    }

    pub fn sql_migration_function(&self) -> &str {
        match &self.sync_type {
            VersionSyncType::Sql(migration_function) => migration_function,
            VersionSyncType::Ts(_) => {
                panic!("Retrieving SQL migration function from a flow version sync.")
            }
        }
    }

    pub fn create_function_query(&self) -> String {
        format!(
            "CREATE FUNCTION {} AS {}",
            self.migration_function_name(),
            self.sql_migration_function()
        )
    }
    pub fn drop_function_query(&self) -> String {
        format!("DROP FUNCTION IF EXISTS {}", self.migration_function_name())
    }

    pub fn create_trigger_query(&self) -> Result<String, ClickhouseError> {
        create_version_sync_trigger_query(self)
    }

    pub fn initial_load_query(&self) -> Result<String, ClickhouseError> {
        create_initial_data_load_query(self)
    }

    pub fn drop_trigger_query(&self) -> String {
        format!(
            "DROP VIEW IF EXISTS {}.{}",
            self.db_name,
            self.migration_trigger_name()
        )
    }
}
