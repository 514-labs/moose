use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::framework::{
    core::infrastructure_map::{PrimitiveSignature, PrimitiveTypes},
    data_model::model::DataModel,
    streaming::model::StreamingFunction,
};

use super::table::Column;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Topic {
    pub version: String,
    pub name: String,
    pub retention_period: Duration,

    pub columns: Vec<Column>,

    pub source_primitive: PrimitiveSignature,
}

impl Topic {
    pub fn from_data_model(data_model: &DataModel) -> Self {
        Topic {
            name: data_model.name.clone(),
            version: data_model.version.clone(),
            // TODO configure this from DataModelConfig
            retention_period: Topic::default_duration(),
            columns: data_model.columns.clone(),
            source_primitive: PrimitiveSignature {
                name: data_model.name.clone(),
                primitive_type: PrimitiveTypes::DataModel,
            },
        }
    }

    pub fn from_migration_function(function: &StreamingFunction) -> (Topic, Topic) {
        let name = |suffix: &str| {
            format!(
                "{}_{}_{}_{}_{}",
                function.source_data_model.name,
                function.source_data_model.version.replace('.', "_"),
                function.target_data_model.name,
                function.target_data_model.version.replace('.', "_"),
                suffix,
            )
        };

        let source_topic = Topic {
            name: name("input"),
            version: function.version.clone(),
            retention_period: Topic::default_duration(),
            columns: function.source_data_model.columns.clone(),
            source_primitive: PrimitiveSignature {
                name: function.id(),
                primitive_type: PrimitiveTypes::Function,
            },
        };

        let target_topic = Topic {
            name: name("output"),
            version: function.version.clone(),
            retention_period: Topic::default_duration(),
            columns: function.target_data_model.columns.clone(),
            source_primitive: PrimitiveSignature {
                name: function.id(),
                primitive_type: PrimitiveTypes::Function,
            },
        };

        (source_topic, target_topic)
    }

    pub fn id(&self) -> String {
        match self.source_primitive.primitive_type {
            // migration functions has versions in the name already
            PrimitiveTypes::Function => self.name.clone(),
            PrimitiveTypes::DataModel
            // the following two possibilities are impossible
            | PrimitiveTypes::DBBlock
            | PrimitiveTypes::ConsumptionAPI => {
                // TODO have a proper version object that standardizes transformations
                format!("{}_{}", self.name, self.version.replace('.', "_"))
            }
        }
    }

    pub fn expanded_display(&self) -> String {
        format!(
            "Topic: {} - Version: {} - Retention Period: {}s",
            self.name,
            self.version,
            self.retention_period.as_secs()
        )
    }

    pub fn short_display(&self) -> String {
        format!("Topic: {} - Version: {}", self.name, self.version)
    }

    fn default_duration() -> Duration {
        Duration::from_secs(60 * 60 * 24 * 7)
    }
}
