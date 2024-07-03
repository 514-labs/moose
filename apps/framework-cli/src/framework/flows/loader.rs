use log::{debug, info, warn};
use regex::{Captures, Regex};
use std::{fs, path::Path};

use crate::{
    framework::data_model::model::DataModelSet,
    project::Project,
    utilities::constants::{PY_FLOW_FILE, TS_FLOW_FILE},
};

use super::model::{Flow, FlowError};

const MIGRATION_REGEX: &str =
    r"^([a-zA-Z0-9_]+)_migrate__([0-9_]+)__(([a-zA-Z0-9_]+)__)?([0-9_]+)$";

/**
 * This function gets the flows as defined by the user for the
 * current version of the moose application.
 */
pub async fn get_all_current_flows(
    project: &Project,
    data_models: &DataModelSet,
) -> Result<Vec<Flow>, FlowError> {
    let flows_path = project.flows_dir();
    get_all_flows(data_models, project.cur_version(), &flows_path).await
}

/**
 * This function should be pointed at the root of the 'flows' directory.
 * Whether this is an old version of the moose app flows (inside the version directory)
 * or the current version of the application.
 *
 * It will assume that all the first children of the directory are the data models
 * sources and the the second children are the data model targets.
 *
 * This also retrieves the migration flows.
 *
 * TODO - handle historical versions. For now this only collects the latest/current version
 * of flows using the latest available topics for each data model.
 */
async fn get_all_flows(
    data_models: &DataModelSet,
    current_version: &str,
    path: &Path,
) -> Result<Vec<Flow>, FlowError> {
    // This should not fail since the regex is hardcoded
    let migration_regex = Regex::new(MIGRATION_REGEX).unwrap();

    let mut flows = vec![];

    for source in fs::read_dir(path)? {
        let source = source?;

        // We check if the file is a migration flow
        if source.metadata()?.is_file() {
            if !source.file_name().to_str().unwrap().ends_with("sql") {
                let potential_flow_file_name = &source
                    .path()
                    .with_extension("")
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                match migration_regex.captures(potential_flow_file_name) {
                    None => {
                        let split: Vec<&str> = potential_flow_file_name.split("__").collect();
                        if split.len() == 2 {
                            let source_data_model_name = split[0];
                            let target_data_model_name = split[1];

                            let source_data_model = if let Some(source_data_model) =
                                data_models.get(source_data_model_name, current_version)
                            {
                                source_data_model
                            } else {
                                warn!(
                                    "Data model {} not found in the data model set",
                                    source_data_model_name
                                );
                                continue;
                            };

                            let target_data_model = if let Some(target_data_model) =
                                data_models.get(target_data_model_name, current_version)
                            {
                                target_data_model
                            } else {
                                warn!(
                                    "Data model {} not found in the data model set",
                                    target_data_model_name
                                );
                                continue;
                            };

                            let flow = Flow {
                                name: potential_flow_file_name.clone(),
                                source_data_model: source_data_model.clone(),
                                target_data_model: target_data_model.clone(),
                                executable: source.path(),
                                version: current_version.to_string(),
                            };
                            flows.push(flow);
                        } else {
                            debug!(
                                "Fragments of file {:?} does not match the convention",
                                source.path()
                            );
                        }
                    }
                    Some(caps) => {
                        let flow = build_migration_flow(
                            &source.file_name().to_string_lossy(),
                            data_models,
                            current_version,
                            caps,
                            &source.path(),
                        );
                        flows.push(flow);
                    }
                }
            }
            // In this case we are currently processing a single file
            // As such we can skip the following steps which are specific
            // to the old naming convention, i.e. nested directory flows
            continue;
        }

        let source_data_model_name = source.file_name().to_string_lossy().to_string();
        let source_data_model = match data_models.get(&source_data_model_name, current_version) {
            Some(model) => model,
            None => {
                warn!(
                    "Data model {} not found in the data model set",
                    source_data_model_name
                );
                continue;
            }
        };

        let target = fs::read_dir(source.path())?;

        for target in target {
            let target = target?;
            let target_data_model_name = target.file_name().to_string_lossy().to_string();
            let target_data_model = match data_models.get(&target_data_model_name, current_version)
            {
                Some(model) => model,
                None => {
                    warn!(
                        "Data model {} not found in the data model set",
                        target_data_model_name
                    );
                    continue;
                }
            };

            for flow_file in fs::read_dir(target.path())? {
                let flow_file = flow_file?;
                let file_name = flow_file.file_name().to_string_lossy().to_string();

                if flow_file.metadata()?.is_file()
                    && (file_name.starts_with(TS_FLOW_FILE) || file_name.starts_with(PY_FLOW_FILE))
                {
                    let flow = Flow {
                        name: file_name.clone(),
                        source_data_model: source_data_model.clone(),
                        target_data_model: target_data_model.clone(),
                        executable: flow_file.path().to_path_buf(),
                        version: current_version.to_string(),
                    };
                    flows.push(flow);

                    // There can only be one flow file per target
                    continue;
                }
            }
        }
    }

    Ok(flows)
}

fn build_migration_flow(
    file_name: &str,
    data_models: &DataModelSet,
    current_version: &str,
    caps: Captures,
    executable: &Path,
) -> Flow {
    info!("Flows Data Captures for migrations {:?}", caps);
    let source_model_name = caps.get(1).unwrap().as_str();
    let source_version = caps.get(2).unwrap().as_str().replace('_', ".");

    let source_data_model = data_models.get(source_model_name, &source_version).unwrap();

    let target_model_name = caps.get(4).map(|m| m.as_str()).unwrap_or(source_model_name);
    let target_version = caps.get(5).unwrap().as_str().replace('_', ".");
    let target_data_model = data_models.get(target_model_name, &target_version).unwrap();

    Flow {
        name: file_name.to_string(),
        source_data_model: source_data_model.clone(),
        target_data_model: target_data_model.clone(),
        executable: executable.to_path_buf(),
        version: current_version.to_string(),
    }
}
