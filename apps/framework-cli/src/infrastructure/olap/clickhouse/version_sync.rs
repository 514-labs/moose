use super::errors::ClickhouseError;
use super::queries::create_initial_data_load_query;
use crate::framework::core::code_loader::{FrameworkObject, FrameworkObjectVersions};
use crate::framework::data_model::model::DataModel;
use crate::framework::languages::SupportedLanguages;
use crate::framework::streaming::generate::generate;
use crate::framework::typescript::templates::TS_BASE_STREAMING_FUNCTION_TEMPLATE;
use crate::infrastructure::olap::clickhouse::model::ClickHouseTable;
use crate::infrastructure::olap::clickhouse::queries::create_version_sync_trigger_query;
use crate::project::Project;
use crate::utilities::constants;
use lazy_static::lazy_static;
use log::debug;
use predicates::ord::ne;
use regex::Regex;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

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

pub struct GeneratedStreamingFunctionMigration {
    pub model_name: String,
    pub source_version: String,
    pub dest_version: String,
    pub code: String,
}

pub fn generate_streaming_function_migration(
    framework_object_versions: &FrameworkObjectVersions,
    version_sync_list: &[VersionSync],
    previous_version: &str,
    project: &Project,
) -> Vec<GeneratedStreamingFunctionMigration> {
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
                let code = generate(project, &old_model.data_model, &fo.data_model);
                res.push(GeneratedStreamingFunctionMigration {
                    model_name: old_model.data_model.name.clone(),
                    source_version: previous_version.to_string(),
                    dest_version: framework_object_versions.current_version.clone(),
                    code,
                })
            }
        }
    }
    res
}

pub fn get_all_version_syncs(
    project: &Project,
    framework_object_versions: &FrameworkObjectVersions,
) -> anyhow::Result<Vec<VersionSync>> {
    let functions_dir = project.streaming_func_dir();
    let mut version_syncs = vec![];
    for entry in std::fs::read_dir(functions_dir)? {
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
                    constants::TYPESCRIPT_FILE_EXTENSION => VersionSyncType::Ts(path.clone()),
                    constants::PYTHON_FILE_EXTENSION => VersionSyncType::Py(path.clone()),
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
                                            dest_data_model: to_table_fo.data_model.clone(),
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
    Py(PathBuf),
}

#[derive(Debug, Clone)]
pub struct VersionSync {
    pub db_name: String,
    pub model_name: String,
    pub source_version: String,
    pub source_table: ClickHouseTable,
    pub source_data_model: DataModel,
    pub dest_data_model: DataModel,
    pub dest_version: String,
    pub dest_table: ClickHouseTable,
    pub sync_type: VersionSyncType,
}

lazy_static! {
    pub static ref VERSION_SYNC_REGEX: Regex =
        //            source_model_name         source     target_model_name   dest_version
        Regex::new(r"^([a-zA-Z0-9_]+)_migrate__([0-9_]+)__(([a-zA-Z0-9_]+)__)?([0-9_]+).(sql|ts|py)$")
            .unwrap();
}

impl VersionSync {
    pub fn source_topic_name(&self) -> String {
        format!(
            "{}_{}",
            self.source_data_model.name.clone(),
            self.source_version.replace('.', "_")
        )
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
            _ => {
                panic!("Retrieving SQL migration function from a streaming function version sync.")
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
            "DROP VIEW IF EXISTS \"{}\".\"{}\"",
            self.db_name,
            self.migration_trigger_name()
        )
    }
}

fn get_ts_import_path(data_model: &DataModel, project: &Project) -> String {
    match data_model
        .abs_file_path
        .strip_prefix(project.old_version_location(&data_model.version).unwrap())
    {
        Ok(file) => {
            format!(
                "versions/{}/{}",
                &data_model.version,
                file.with_extension("").to_string_lossy()
            )
        }
        Err(_) => {
            assert_eq!(data_model.version, project.cur_version());

            format!(
                "datamodels/{}",
                data_model
                    .abs_file_path
                    .with_extension("")
                    .strip_prefix(project.data_models_dir())
                    .unwrap()
                    .to_string_lossy()
            )
        }
    }
}
