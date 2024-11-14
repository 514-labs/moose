use super::table::Column;
use crate::framework::versions::Version;
use crate::framework::{
    core::infrastructure_map::{PrimitiveSignature, PrimitiveTypes},
    data_model::model::DataModel,
    streaming::model::StreamingFunction,
};
use protobuf::MessageField;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::proto::infrastructure_map::Topic as ProtoTopic;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Topic {
    pub version: Version<'static>,
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

    pub fn from_migration_function(function: &StreamingFunction) -> (Topic, Option<Topic>) {
        let name = |suffix: &str| {
            format!(
                "{}_{}_{}_{}",
                function.source_data_model.name,
                function.source_data_model.version.as_suffix(),
                function.id(),
                suffix,
            )
        };

        let source_topic = Topic {
            name: name("input"),
            version: function.source_data_model.version.clone(),
            retention_period: Topic::default_duration(),
            columns: function.source_data_model.columns.clone(),
            source_primitive: PrimitiveSignature {
                name: function.id(),
                primitive_type: PrimitiveTypes::Function,
            },
        };

        let target_topic = function
            .target_data_model
            .as_ref()
            .map(|target_data_model| Topic {
                name: name("output"),
                version: target_data_model.version.clone(),
                retention_period: Topic::default_duration(),
                columns: target_data_model.columns.clone(),
                source_primitive: PrimitiveSignature {
                    name: function.id(),
                    primitive_type: PrimitiveTypes::Function,
                },
            });

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
                format!("{}_{}", self.name, self.version.as_suffix())
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

    pub fn to_proto(&self) -> ProtoTopic {
        ProtoTopic {
            version: self.version.to_string(),
            name: self.name.clone(),
            retention_period: MessageField::some(
                protobuf::well_known_types::duration::Duration::from(self.retention_period),
            ),
            columns: self.columns.iter().map(|c| c.to_proto()).collect(),
            source_primitive: MessageField::some(self.source_primitive.to_proto()),
            special_fields: Default::default(),
        }
    }
}
