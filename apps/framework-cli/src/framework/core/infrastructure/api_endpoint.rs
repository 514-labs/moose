use super::{topic::Topic, DataLineage, InfrastructureSignature};
use crate::framework::versions::Version;
use crate::framework::{
    consumption::model::{ConsumptionQueryParam, EndpointFile},
    core::infrastructure_map::{PrimitiveSignature, PrimitiveTypes},
    data_model::{config::EndpointIngestionFormat, model::DataModel},
};
use crate::proto::infrastructure_map::api_endpoint::Api_type as ProtoApiType;
use crate::proto::infrastructure_map::Method as ProtoMethod;
use crate::proto::infrastructure_map::{
    ApiEndpoint as ProtoApiEndpoint, EgressDetails, EndpointIngestionFormat as ProtoIngressFormat,
    IngressDetails,
};
use protobuf::{EnumOrUnknown, MessageField};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::path::PathBuf;

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
    EGRESS {
        query_params: Vec<ConsumptionQueryParam>,
        #[serde(default)]
        output_schema: Value,
    },
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
    #[serde(deserialize_with = "APIType::backwards_compatible_deserialize")]
    pub api_type: APIType,
    pub path: PathBuf,
    pub method: Method,

    pub version: Version,
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
                .join(data_model.version.as_str()),
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
            "{}_{}_{}",
            match self.api_type {
                APIType::INGRESS { .. } => "INGRESS",
                APIType::EGRESS { .. } => "EGRESS",
            },
            self.name,
            self.version.as_suffix()
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
            APIType::EGRESS { .. } => None,
        }
    }

    pub fn short_display(&self) -> String {
        format!("API Endpoint: {} - Version: {}", self.name, self.version)
    }

    pub fn to_proto(&self) -> ProtoApiEndpoint {
        ProtoApiEndpoint {
            name: self.name.clone(),
            api_type: Some(self.api_type.to_proto()),
            path: self.path.to_string_lossy().to_string(),
            method: EnumOrUnknown::new(self.method.to_proto()),
            version: self.version.to_string(),
            source_primitive: MessageField::some(self.source_primitive.to_proto()),
            special_fields: Default::default(),
        }
    }

    pub fn from_proto(proto: ProtoApiEndpoint) -> Self {
        ApiEndpoint {
            name: proto.name,
            api_type: APIType::from_proto(proto.api_type.unwrap()),
            path: PathBuf::from(proto.path),
            method: Method::from_proto(
                proto
                    .method
                    .enum_value()
                    .expect("Invalid method enum value"),
            ),
            version: Version::from_string(proto.version),
            source_primitive: PrimitiveSignature::from_proto(proto.source_primitive.unwrap()),
        }
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
            api_type: APIType::EGRESS {
                query_params: value.query_params,
                output_schema: value.output_schema,
            },
            path: value.path.clone(),
            method: Method::GET,
            version: Version::from_string("0.0.0".to_string()),
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
            APIType::EGRESS { .. } => vec![],
        }
    }
}

impl APIType {
    fn to_proto(&self) -> ProtoApiType {
        match self {
            APIType::INGRESS {
                target_topic,
                data_model: _data_model,
                format,
            } => ProtoApiType::Ingress(IngressDetails {
                target_topic: target_topic.clone(),
                format: EnumOrUnknown::new(match format {
                    EndpointIngestionFormat::Json => ProtoIngressFormat::JSON,
                    EndpointIngestionFormat::JsonArray => ProtoIngressFormat::JSON_ARRAY,
                }),
                special_fields: Default::default(),
            }),
            APIType::EGRESS {
                query_params,
                output_schema,
            } => ProtoApiType::Egress(EgressDetails {
                query_params: query_params.iter().map(|param| param.to_proto()).collect(),
                output_schema: output_schema.to_string(),
                special_fields: Default::default(),
            }),
        }
    }

    fn backwards_compatible_deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Value::deserialize(deserializer)?;
        match json {
            Value::String(s) if s == "EGRESS" => Ok(APIType::EGRESS {
                query_params: vec![],
                output_schema: Value::Null,
            }),
            _ => serde_json::from_value(json).map_err(D::Error::custom),
        }
    }

    pub fn from_proto(proto: ProtoApiType) -> Self {
        match proto {
            ProtoApiType::Ingress(details) => APIType::INGRESS {
                target_topic: details.target_topic,
                data_model: None,
                format: match details
                    .format
                    .enum_value()
                    .expect("Invalid format enum value")
                {
                    ProtoIngressFormat::JSON => EndpointIngestionFormat::Json,
                    ProtoIngressFormat::JSON_ARRAY => EndpointIngestionFormat::JsonArray,
                },
            },
            ProtoApiType::Egress(details) => APIType::EGRESS {
                query_params: details
                    .query_params
                    .into_iter()
                    .map(ConsumptionQueryParam::from_proto)
                    .collect(),
                output_schema: serde_json::from_str(&details.output_schema).unwrap_or_default(),
            },
        }
    }
}

impl Method {
    fn to_proto(&self) -> ProtoMethod {
        match self {
            Method::GET => ProtoMethod::GET,
            Method::POST => ProtoMethod::POST,
            Method::PUT => ProtoMethod::PUT,
            Method::DELETE => ProtoMethod::DELETE,
        }
    }

    pub fn from_proto(proto: ProtoMethod) -> Self {
        match proto {
            ProtoMethod::GET => Method::GET,
            ProtoMethod::POST => Method::POST,
            ProtoMethod::PUT => Method::PUT,
            ProtoMethod::DELETE => Method::DELETE,
        }
    }
}
