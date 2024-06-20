use hyper::http::Method;
use std::{collections::HashMap, time::Duration};

use crate::framework::data_model::{
    config::EndpointIngestionFormat,
    schema::{DataModel, Table},
};

use super::primitive_map::PrimitiveMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Topic {
    pub version: String,
    pub name: String,

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

    pub fn id(&self) -> String {
        // TODO have a proper version object that standardizes transformations
        format!("{}_{}", self.name, self.version.replace('.', "_"))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum APIType {
    INGRESS,
    EGRESS,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveTypes {
    DataModel,
    Function,
    DBBlock,
    ConsumptionAPI,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveSignature {
    pub name: String,
    pub primitive_type: PrimitiveTypes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiEndpoint {
    pub name: String,
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

    pub fn id(&self) -> String {
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone)]
pub enum TopicChange {
    Added(Topic),
    Removed(Topic),
    Updated { before: Topic, after: Topic },
}

#[derive(Debug, Clone)]
pub enum ApiEndpointChange {
    Added(ApiEndpoint),
    Removed(ApiEndpoint),
    Updated {
        before: ApiEndpoint,
        after: ApiEndpoint,
    },
}

#[derive(Debug, Clone)]
pub enum TableChange {
    Added(Table),
    Removed(Table),
    Updated { before: Table, after: Table },
}

#[derive(Debug, Clone)]
pub enum TopicToTableSyncProcessChange {
    Added(TopicToTableSyncProcess),
    Removed(TopicToTableSyncProcess),
    Updated {
        before: TopicToTableSyncProcess,
        after: TopicToTableSyncProcess,
    },
}

#[derive(Debug, Clone)]
pub enum InfraChange {
    Topic(TopicChange),
    ApiEndpoint(ApiEndpointChange),
    Table(TableChange),
    TopicToTableSyncProcess(TopicToTableSyncProcessChange),
}

#[derive(Debug, Clone)]
pub struct InfrastructureMap {
    // primitive_map: PrimitiveMap,
    pub topics: HashMap<String, Topic>,
    pub api_endpoints: HashMap<String, ApiEndpoint>,
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

    // The current implementation is simple in which it goes over some of the
    // changes several times. Something could be done to optimize this.
    // There is probably a way to make this also more generic so that we don't have very
    // similar code for each type of change.
    pub fn diff(&self, target_map: &InfrastructureMap) -> Vec<InfraChange> {
        let mut changes = vec![];

        // =================================================================
        //                              Topics
        // =================================================================

        for (id, topic) in &self.topics {
            if let Some(target_topic) = target_map.topics.get(id) {
                if topic != target_topic {
                    changes.push(InfraChange::Topic(TopicChange::Updated {
                        before: topic.clone(),
                        after: target_topic.clone(),
                    }));
                }
            } else {
                changes.push(InfraChange::Topic(TopicChange::Removed(topic.clone())));
            }
        }

        for (id, topic) in &target_map.topics {
            if !self.topics.contains_key(id) {
                changes.push(InfraChange::Topic(TopicChange::Added(topic.clone())));
            }
        }

        // =================================================================
        //                              API Endpoints
        // =================================================================

        for (id, api_endpoint) in &self.api_endpoints {
            if let Some(target_api_endpoint) = target_map.api_endpoints.get(id) {
                if api_endpoint != target_api_endpoint {
                    changes.push(InfraChange::ApiEndpoint(ApiEndpointChange::Updated {
                        before: api_endpoint.clone(),
                        after: target_api_endpoint.clone(),
                    }));
                }
            } else {
                changes.push(InfraChange::ApiEndpoint(ApiEndpointChange::Removed(
                    api_endpoint.clone(),
                )));
            }
        }

        for (id, api_endpoint) in &target_map.api_endpoints {
            if !self.api_endpoints.contains_key(id) {
                changes.push(InfraChange::ApiEndpoint(ApiEndpointChange::Added(
                    api_endpoint.clone(),
                )));
            }
        }

        // =================================================================
        //                              Tables
        // =================================================================

        for (id, table) in &self.tables {
            if let Some(target_table) = target_map.tables.get(id) {
                if table != target_table {
                    changes.push(InfraChange::Table(TableChange::Updated {
                        before: table.clone(),
                        after: target_table.clone(),
                    }));
                }
            } else {
                changes.push(InfraChange::Table(TableChange::Removed(table.clone())));
            }
        }

        for (id, table) in &target_map.tables {
            if !self.tables.contains_key(id) {
                changes.push(InfraChange::Table(TableChange::Added(table.clone())));
            }
        }

        // =================================================================
        //                              Topic to Table Sync Processes
        // =================================================================

        for (id, topic_to_table_sync_process) in &self.topic_to_table_sync_processes {
            if let Some(target_topic_to_table_sync_process) =
                target_map.topic_to_table_sync_processes.get(id)
            {
                if topic_to_table_sync_process != target_topic_to_table_sync_process {
                    changes.push(InfraChange::TopicToTableSyncProcess(
                        TopicToTableSyncProcessChange::Updated {
                            before: topic_to_table_sync_process.clone(),
                            after: target_topic_to_table_sync_process.clone(),
                        },
                    ));
                }
            } else {
                changes.push(InfraChange::TopicToTableSyncProcess(
                    TopicToTableSyncProcessChange::Removed(topic_to_table_sync_process.clone()),
                ));
            }
        }

        for (id, topic_to_table_sync_process) in &target_map.topic_to_table_sync_processes {
            if !self.topic_to_table_sync_processes.contains_key(id) {
                changes.push(InfraChange::TopicToTableSyncProcess(
                    TopicToTableSyncProcessChange::Added(topic_to_table_sync_process.clone()),
                ));
            }
        }

        changes
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::{
        framework::{
            core::primitive_map::PrimitiveMap, data_model::schema::DataModel,
            languages::SupportedLanguages,
        },
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

    #[test]
    #[ignore]
    fn test_infra_diff_map() {
        let project = Project::new(
            Path::new("/Users/nicolas/code/514/test"),
            "test".to_string(),
            SupportedLanguages::Typescript,
        );
        let primitive_map = PrimitiveMap::load(&project).unwrap();
        let mut new_target_primitive_map = primitive_map.clone();

        let new_data_model = DataModel {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            config: Default::default(),
            columns: vec![],
            file_path: PathBuf::new(),
        };
        // Making some changes to the map
        new_target_primitive_map
            .datamodels
            .insert(PathBuf::new(), vec![new_data_model]);
        let to_change = PathBuf::from("/Users/nicolas/code/514/test/app/datamodels/models.ts");
        let to_be_changed = new_target_primitive_map
            .datamodels
            .get_mut(&to_change)
            .unwrap();
        to_be_changed[0].columns = vec![];
        to_be_changed.pop();

        println!("Base Primitve Map: {:?} \n", primitive_map);
        println!("Target Primitive Map {:?} \n", new_target_primitive_map);

        let infra_map = super::InfrastructureMap::new(primitive_map);
        let new_infra_map = super::InfrastructureMap::new(new_target_primitive_map);

        let diffs = infra_map.diff(&new_infra_map);

        print!("Diffs: {:?}", diffs);
    }
}
