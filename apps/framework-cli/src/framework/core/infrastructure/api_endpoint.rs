use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::framework::{
    consumption::model::EndpointFile,
    core::infrastructure_map::{PrimitiveSignature, PrimitiveTypes},
    data_model::{config::EndpointIngestionFormat, model::DataModel},
};

use super::{topic::Topic, DataLineage, InfrastructureSignature};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum APIType {
    INGRESS {
        target_topic: String,
        // in previous versions this is not stored,
        // so deserialization may fail if the value is not optional
        // our code does not depend on the stored field
        data_model: Option<DataModel>,
        format: EndpointIngestionFormat,
    },
    EGRESS,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub name: String,
    pub api_type: APIType,
    pub path: PathBuf,
    pub method: Method,

    pub version: String,
    pub source_primitive: PrimitiveSignature,
}

impl ApiEndpoint {
    pub fn from_data_model(data_model: &DataModel, topic: &Topic) -> Self {
        ApiEndpoint {
            name: data_model.name.clone(),
            api_type: APIType::INGRESS {
                target_topic: topic.id(),
                data_model: Some(data_model.clone()),
                format: data_model.config.ingestion.format,
            },
            // This implementation is actually removing the functionality of nestedness of paths in
            // data model to change the ingest path. However, we are changing how this works with an
            // explicit in ingest path and we have not seen people use that functionality yet.
            path: PathBuf::from("ingest")
                .join(data_model.name.clone())
                .join(data_model.version.clone()),
            method: Method::POST,
            version: data_model.version.clone(),
            source_primitive: PrimitiveSignature {
                name: data_model.name.clone(),
                primitive_type: PrimitiveTypes::DataModel,
            },
        }
    }

    pub fn id(&self) -> String {
        // TODO have a proper version object that standardizes transformations
        format!(
            "{:?}_{}_{}",
            self.api_type,
            self.name,
            self.version.replace('.', "_")
        )
    }

    pub fn expanded_display(&self) -> String {
        format!(
            "API Endpoint: {} - Version: {} - Path: {} - Method: {:?} - Format: {:?}",
            self.name,
            self.version,
            self.path.to_string_lossy(),
            self.method,
            self.format(),
        )
    }

    fn format(&self) -> Option<EndpointIngestionFormat> {
        match self.api_type {
            APIType::INGRESS { format, .. } => Some(format),
            APIType::EGRESS => None,
        }
    }

    pub fn short_display(&self) -> String {
        format!("API Endpoint: {} - Version: {}", self.name, self.version)
    }
}

impl From<EndpointFile> for ApiEndpoint {
    fn from(value: EndpointFile) -> Self {
        ApiEndpoint {
            name: value
                .path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            api_type: APIType::EGRESS,
            path: value.path.clone(),
            method: Method::GET,
            version: "0.0.0".to_string(),
            source_primitive: PrimitiveSignature {
                name: value.path.to_string_lossy().to_string(),
                primitive_type: PrimitiveTypes::ConsumptionAPI,
            },
        }
    }
}

impl DataLineage for ApiEndpoint {
    fn receives_data_from(&self) -> Vec<InfrastructureSignature> {
        vec![]
    }

    fn pulls_data_from(&self) -> Vec<InfrastructureSignature> {
        vec![]
    }

    fn pushes_data_to(&self) -> Vec<InfrastructureSignature> {
        match &self.api_type {
            APIType::INGRESS { target_topic, .. } => {
                vec![InfrastructureSignature::Topic {
                    id: target_topic.clone(),
                }]
            }
            APIType::EGRESS => vec![],
        }
    }
}
