use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::framework::{
    core::infrastructure_map::{PrimitiveSignature, PrimitiveTypes},
    data_model::schema::DataModel,
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
            retention_period: Duration::from_secs(60 * 60 * 24 * 7),
            columns: data_model.columns.clone(),
            source_primitive: PrimitiveSignature {
                name: data_model.name.clone(),
                primitive_type: PrimitiveTypes::DataModel,
            },
        }
    }

    pub fn id(&self) -> String {
        // TODO have a proper version object that standardizes transformations
        format!("{}_{}", self.name, self.version.replace('.', "_"))
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
}
