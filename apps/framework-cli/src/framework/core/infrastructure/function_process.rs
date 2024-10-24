use crate::{
    framework::{
        core::infrastructure_map::{PrimitiveSignature, PrimitiveTypes},
        streaming::model::StreamingFunction,
    },
    infrastructure::stream::redpanda::RedpandaConfig,
    utilities::constants::{PYTHON_FILE_EXTENSION, TYPESCRIPT_FILE_EXTENSION},
};
use itertools::sorted;
use protobuf::MessageField;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

use super::{table::Column, topic::Topic};

use crate::proto::infrastructure_map::FunctionProcess as ProtoFunctionProcess;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionProcess {
    // The name used here is the name of the file that contains the function
    // since we have the current assumption that there is 1 function per file
    // we can use the file name as the name of the function.
    // We don't currently do checks across functions for unicities but we should
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
    pub fn from_function(function: &StreamingFunction, topics: &[String]) -> Self {
        FunctionProcess {
            name: function.name.clone(),

            // This probably should be a reference to the topic ingested from the
            // infra map instead of using a convention for the topic name.
            // Leaving it as is for compatibility with the current code.
            source_topic: get_latest_topic(topics, &function.source_data_model.name)
                .unwrap_or_else(|| function.source_data_model.name.clone()),
            source_columns: function.source_data_model.columns.clone(),

            // This probably should be a reference to the topic ingested from the
            // infra map instead of using a convention for the topic name.
            // Leaving it as is for compatibility with the current code.
            target_topic: function
                .target_data_model
                .as_ref()
                .map(|target_model| {
                    get_latest_topic(topics, &target_model.name)
                        .unwrap_or_else(|| target_model.name.clone())
                })
                .unwrap_or_default(),

            target_columns: function
                .target_data_model
                .as_ref()
                .map(|target_model| target_model.columns.clone())
                .unwrap_or_default(),
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

    pub fn from_migration_function(
        function: &StreamingFunction,
        source_topic: &Topic,
        target_topic: &Topic,
    ) -> Self {
        FunctionProcess {
            name: function.name.clone(),

            source_topic: source_topic.name.clone(),
            source_columns: function.source_data_model.columns.clone(),

            target_topic: target_topic.name.clone(),

            target_columns: function.target_data_model.as_ref().unwrap().columns.clone(),
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

    pub fn to_proto(&self) -> ProtoFunctionProcess {
        ProtoFunctionProcess {
            name: self.name.clone(),
            source_topic: self.source_topic.clone(),
            source_columns: self.source_columns.iter().map(|c| c.to_proto()).collect(),
            target_topic: self.target_topic.clone(),
            target_topic_config: self.target_topic_config.clone(),
            target_columns: self.target_columns.iter().map(|c| c.to_proto()).collect(),
            executable: self.executable.to_str().unwrap_or_default().to_string(),
            version: self.version.clone(),
            source_primitive: MessageField::some(self.source_primitive.to_proto()),
            special_fields: Default::default(),
        }
    }
}

/**
 * This function retrieves the latest topic for a given data model
 * This should be eventually retired for proper DCM management for functions.
 */
fn get_latest_topic(topics: &[String], data_model: &str) -> Option<String> {
    // Ths algorithm is not super efficient. We probably should have a
    // good way to retrieve the topics for a given data model from the state
    sorted(
        // FIXME: this will put version no. `3.11` before `3.9`
        topics
            .iter()
            .filter(|&topic| {
                RedpandaConfig::get_topic_without_namespace(topic).starts_with(data_model)
            })
            .collect::<Vec<&String>>(),
    )
    .last()
    .map(|topic| topic.to_string())
}
