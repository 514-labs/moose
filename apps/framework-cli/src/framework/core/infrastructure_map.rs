use hyper::http::Method;
use std::{collections::HashMap, time::Duration};

use crate::framework::data_model::{
    config::EndpointIngestionFormat,
    schema::{DataModel, Table},
};

use super::primitive_map::PrimitiveMap;

type TopicId = String;
type APIId = String;

#[derive(Debug)]
pub struct Topic {
    pub version: String,
    pub name: TopicId,

    pub retention_period: Duration,
    pub source_primitive: PrimitiveSignature,
}

impl Topic {
    pub fn from_data_model(data_model: &DataModel) -> Self {
        Topic {
            name: data_model.name.clone(),
            version: data_model.version.clone(),
            // TODO configure this from DataModelConfig
            retention_period: Duration::from_secs(60 * 60 * 24 * 7),
            source_primitive: PrimitiveSignature {
                name: data_model.name.clone(),
                primitive_type: PrimitiveTypes::DataModel,
            },
        }
    }

    pub fn id(&self) -> TopicId {
        // TODO have a proper version object that standardizes transformations
        format!("{}_{}", self.name, self.version.replace('.', "_"))
    }
}

#[derive(Debug)]
pub enum APIType {
    INGRESS,
    EGRESS,
}

#[derive(Debug, Clone)]
pub enum PrimitiveTypes {
    DataModel,
    Function,
    DBBlock,
    ConsumptionAPI,
}

#[derive(Debug, Clone)]
pub struct PrimitiveSignature {
    pub name: String,
    pub primitive_type: PrimitiveTypes,
}

#[derive(Debug)]
pub struct ApiEndpoint {
    pub name: APIId,
    pub api_type: APIType,
    pub path: String,
    pub method: Method,
    pub format: EndpointIngestionFormat,

    pub version: String,
    pub source_primitive: PrimitiveSignature,
}

impl ApiEndpoint {
    pub fn from_data_model(data_model: &DataModel) -> Self {
        ApiEndpoint {
            name: data_model.name.clone(),
            api_type: APIType::INGRESS,
            path: format!("/ingest/{}/{}", data_model.name, data_model.version),
            method: Method::POST,
            format: data_model.config.ingestion.format.clone(),
            version: data_model.version.clone(),
            source_primitive: PrimitiveSignature {
                name: data_model.name.clone(),
                primitive_type: PrimitiveTypes::DataModel,
            },
        }
    }

    pub fn id(&self) -> APIId {
        // TODO have a proper version object that standardizes transformations
        format!(
            "{:?}_{}_{}",
            self.api_type,
            self.name,
            self.version.replace('.', "_")
        )
    }
}

// pub struct TopicSyncProcess {
//     source_topic_id: String,
//     destination_topic_id: String,

//     pub version: String,
//     pub source_primitive: PrimitiveSignature,
// }

#[derive(Debug)]
pub struct TopicToTableSyncProcess {
    source_topic_id: String,
    destination_table_id: String,

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
            destination_table_id: table.id(),
            version: topic.version.clone(),
            source_primitive: topic.source_primitive.clone(),
        }
    }

    pub fn id(&self) -> String {
        format!(
            "{}_{}_{}",
            self.source_topic_id,
            self.destination_table_id,
            self.version.replace('.', "_")
        )
    }
}

#[derive(Debug)]
pub struct InfrastructureMap {
    // primitive_map: PrimitiveMap,
    pub topics: HashMap<TopicId, Topic>,
    pub api_endpoints: HashMap<APIId, ApiEndpoint>,
    pub tables: HashMap<String, Table>,

    pub topic_to_table_sync_processes: HashMap<String, TopicToTableSyncProcess>,
}

impl InfrastructureMap {
    pub fn new(primitive_map: PrimitiveMap) -> InfrastructureMap {
        let mut tables = HashMap::new();
        let mut topics = HashMap::new();
        let mut api_endpoints = HashMap::new();
        let mut topic_to_table_sync_processes = HashMap::new();

        for data_model in primitive_map.data_models_iter() {
            let topic = Topic::from_data_model(data_model);
            let api_endpoint = ApiEndpoint::from_data_model(data_model);

            if data_model.config.storage.enabled {
                let table = data_model.to_table();
                let topic_to_table_sync_process = TopicToTableSyncProcess::new(&topic, &table);

                tables.insert(table.id(), table);
                topic_to_table_sync_processes.insert(
                    topic_to_table_sync_process.id(),
                    topic_to_table_sync_process,
                );
            }

            topics.insert(topic.id(), topic);
            api_endpoints.insert(api_endpoint.id(), api_endpoint);
        }

        InfrastructureMap {
            // primitive_map,
            topics,
            api_endpoints,
            topic_to_table_sync_processes,
            tables,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{
        framework::{core::primitive_map::PrimitiveMap, languages::SupportedLanguages},
        project::Project,
    };

    #[test]
    #[ignore]
    fn test_infra_map() {
        let project = Project::new(
            Path::new("/Users/nicolas/code/514/test"),
            "test".to_string(),
            SupportedLanguages::Typescript,
        );
        let primitive_map = PrimitiveMap::load(&project);
        println!("{:?}", primitive_map);
        assert!(primitive_map.is_ok());

        let infra_map = super::InfrastructureMap::new(primitive_map.unwrap());
        println!("{:?}", infra_map);
    }
}
