use serde::{Deserialize, Serialize};

use crate::framework::core::infrastructure_map::PrimitiveSignature;

use super::{
    table::{Column, Table},
    topic::Topic,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopicToTableSyncProcess {
    pub source_topic_id: String,
    pub target_table_id: String,

    pub columns: Vec<Column>,

    pub version: String,
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
