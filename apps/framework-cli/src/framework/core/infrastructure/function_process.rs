use itertools::sorted;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

use crate::{
    framework::{
        core::infrastructure_map::{PrimitiveSignature, PrimitiveTypes},
        flows::model::Flow,
    },
    utilities::constants::{PYTHON_FILE_EXTENSION, TYPESCRIPT_FILE_EXTENSION},
};

use super::{table::Column, topic::Topic};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionProcess {
    // The name used here is the name of the file that contains the function
    // since we have the current assumption that there is 1 function per file
    // we can use the file name as the name of the function.
    // We don't currently do checks accross flows for unicities but we should
    pub name: String,

    pub source_topic: String,
    pub source_columns: Vec<Column>,

    pub target_topic: String,
    pub target_topic_config: HashMap<String, String>,
    pub target_columns: Vec<Column>,

    pub executable: PathBuf,

    pub version: String,
    pub source_primitive: PrimitiveSignature,
}

impl FunctionProcess {
    pub fn from_functon(function: &Flow, topics: &[String]) -> Self {
        FunctionProcess {
            name: function.name.clone(),

            // This probably should be a reference to the topic ingested from the
            // infra map insteaad of using a convention for the topic name.
            // Leaving it as is for compatibility with the current code.
            source_topic: get_latest_topic(topics, &function.source_data_model.name)
                .unwrap_or_else(|| function.source_data_model.name.clone()),
            source_columns: function.source_data_model.columns.clone(),

            // This probably should be a reference to the topic ingested from the
            // infra map insteaad of using a convention for the topic name.
            // Leaving it as is for compatibility with the current code.
            target_topic: get_latest_topic(topics, &function.target_data_model.name)
                .unwrap_or_else(|| function.target_data_model.name.clone()),

            target_columns: function.target_data_model.columns.clone(),
            target_topic_config: HashMap::from([
                ("max.message.bytes".to_string(), (1024 * 1024).to_string()),
                ("message.max.bytes".to_string(), (1024 * 1024).to_string()),
            ]),

            executable: function.executable.clone(),

            version: function.version.clone(),
            source_primitive: PrimitiveSignature {
                name: function.name.clone(),
                primitive_type: PrimitiveTypes::Function,
            },
        }
    }

    pub fn from_migration_functon(
        function: &Flow,
        source_topic: &Topic,
        target_topic: &Topic,
    ) -> Self {
        FunctionProcess {
            name: function.name.clone(),

            source_topic: source_topic.name.clone(),
            source_columns: function.source_data_model.columns.clone(),

            target_topic: target_topic.name.clone(),

            target_columns: function.target_data_model.columns.clone(),
            target_topic_config: HashMap::from([
                ("max.message.bytes".to_string(), (1024 * 1024).to_string()),
                ("message.max.bytes".to_string(), (1024 * 1024).to_string()),
            ]),

            executable: function.executable.clone(),

            version: function.version.clone(),
            source_primitive: PrimitiveSignature {
                name: function.name.clone(),
                primitive_type: PrimitiveTypes::Function,
            },
        }
    }

    pub fn is_ts_function_process(&self) -> bool {
        self.executable.extension().unwrap().to_str().unwrap() == TYPESCRIPT_FILE_EXTENSION
    }

    pub fn is_py_function_process(&self) -> bool {
        self.executable.extension().unwrap().to_str().unwrap() == PYTHON_FILE_EXTENSION
    }

    pub fn id(&self) -> String {
        format!(
            "{}_{}_{}_{}",
            self.name, self.source_topic, self.target_topic, self.version
        )
    }

    pub fn expanded_display(&self) -> String {
        format!(
            "Reloading Function: from topic {} to topic {} - Version: {}",
            self.source_topic, self.target_topic, self.version
        )
    }

    pub fn short_display(&self) -> String {
        self.expanded_display()
    }

    pub fn target_topic_config_json(&self) -> String {
        serde_json::to_string(&self.target_topic_config).unwrap()
    }
}

/**
 * This function retrieves the latest topic for a given data model
 * This should be eventually retired for proper DCM management for flows.
 */
fn get_latest_topic(topics: &[String], data_model: &str) -> Option<String> {
    // Ths algorithm is not super efficient. We probably should have a
    // good way to retrieve the topics for a given data model from the state
    sorted(
        topics
            .iter()
            .filter(|&topic| topic.starts_with(data_model))
            .collect::<Vec<&String>>(),
    )
    .last()
    .map(|topic| topic.to_string())
}
