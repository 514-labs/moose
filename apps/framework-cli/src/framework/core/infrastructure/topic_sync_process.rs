use serde::{Deserialize, Serialize};

use super::{
    table::{Column, Table},
    topic::Topic,
};
use crate::framework::core::infrastructure_map::{PrimitiveSignature, PrimitiveTypes};
use crate::framework::streaming::model::StreamingFunction;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopicToTableSyncProcess {
    pub source_topic_id: String,
    pub target_table_id: String,

    pub columns: Vec<Column>,

    pub version: String,
    pub source_primitive: PrimitiveSignature,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopicToTopicSyncProcess {
    pub source_topic_id: String,
    pub target_topic_id: String,

    pub source_primitive: PrimitiveSignature,
}

impl TopicToTableSyncProcess {
    pub fn new(topic: &Topic, table: &Table) -> Self {
        if topic.version != table.version {
            panic!("Version mismatch between topic and table")
        }

        TopicToTableSyncProcess {
            source_topic_id: topic.id(),
            columns: topic.columns.clone(),
            // TODO - MIGRATE - should become id() when we migrate over to the new core
            target_table_id: table.name.clone(),
            version: topic.version.clone(),
            source_primitive: topic.source_primitive.clone(),
        }
    }

    pub fn id(&self) -> String {
        format!(
            "{}_{}_{}",
            self.source_topic_id,
            self.target_table_id,
            self.version.replace('.', "_")
        )
    }

    pub fn expanded_display(&self) -> String {
        format!(
            "Topic to Table Sync Process: {} -> {}",
            self.source_topic_id, self.target_table_id
        )
    }

    pub fn short_display(&self) -> String {
        format!(
            "Topic to Table Sync Process: {} -> {}",
            self.source_topic_id, self.target_table_id
        )
    }
}

impl TopicToTopicSyncProcess {
    pub fn from_migration_function(function: &StreamingFunction) -> Self {
        let source_topic = Topic::from_data_model(&function.source_data_model);
        let (source_for_func, _) = Topic::from_migration_function(function);
        TopicToTopicSyncProcess {
            source_topic_id: source_topic.id(),
            target_topic_id: source_for_func.id(),
            source_primitive: PrimitiveSignature {
                name: function.id(),
                primitive_type: PrimitiveTypes::Function,
            },
        }
    }

    pub fn id(&self) -> String {
        self.target_topic_id.to_string()
    }

    pub fn expanded_display(&self) -> String {
        format!(
            "Topic to Topic Sync Process: {} -> {}",
            self.source_topic_id, self.target_topic_id
        )
    }

    pub fn short_display(&self) -> String {
        format!(
            "Topic to Topic Sync Process: {} -> {}",
            self.source_topic_id, self.target_topic_id
        )
    }
}
