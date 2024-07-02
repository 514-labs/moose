use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::infrastructure::api_endpoint::ApiEndpoint;
use super::infrastructure::function_process::FunctionProcess;
use super::infrastructure::table::Table;
use super::infrastructure::topic::Topic;
use super::infrastructure::topic_to_table_sync_process::TopicToTableSyncProcess;
use super::primitive_map::PrimitiveMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveTypes {
    DataModel,
    Function,
    DBBlock,
    ConsumptionAPI,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrimitiveSignature {
    pub name: String,
    pub primitive_type: PrimitiveTypes,
}

#[derive(Debug, Clone)]
pub enum Change<T> {
    Added(T),
    Removed(T),
    Updated { before: T, after: T },
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum InfraChange {
    Olap(OlapChange),
    Streaming(StreamingChange),
    Api(ApiChange),
    Process(ProcessChange),
}

#[derive(Debug, Clone)]
pub enum OlapChange {
    Table(Change<Table>),
}

#[derive(Debug, Clone)]
pub enum StreamingChange {
    Topic(Change<Topic>),
}

#[derive(Debug, Clone)]
pub enum ApiChange {
    ApiEndpoint(Change<ApiEndpoint>),
}

#[derive(Debug, Clone)]
pub enum ProcessChange {
    TopicToTableSyncProcess(Change<TopicToTableSyncProcess>),
    FunctionProcess(Change<FunctionProcess>),
}

#[derive(Debug, Clone, Default)]
pub struct InfraChanges {
    pub olap_changes: Vec<OlapChange>,
    pub processes_changes: Vec<ProcessChange>,
    pub api_changes: Vec<ApiChange>,
    pub streaming_engine_changes: Vec<StreamingChange>,
}

impl InfraChanges {
    pub fn is_empty(&self) -> bool {
        self.olap_changes.is_empty()
            && self.processes_changes.is_empty()
            && self.api_changes.is_empty()
            && self.streaming_engine_changes.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureMap {
    // primitive_map: PrimitiveMap,
    pub topics: HashMap<String, Topic>,
    pub api_endpoints: HashMap<String, ApiEndpoint>,
    pub tables: HashMap<String, Table>,

    pub topic_to_table_sync_processes: HashMap<String, TopicToTableSyncProcess>,
    pub function_processes: HashMap<String, FunctionProcess>,
}

impl InfrastructureMap {
    pub fn new(primitive_map: PrimitiveMap) -> InfrastructureMap {
        let mut tables = HashMap::new();
        let mut topics = HashMap::new();
        let mut api_endpoints = HashMap::new();
        let mut topic_to_table_sync_processes = HashMap::new();
        let mut function_processes = HashMap::new();

        for data_model in primitive_map.data_models_iter() {
            let topic = Topic::from_data_model(data_model);
            let api_endpoint = ApiEndpoint::from_data_model(data_model, &topic);

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

        for function in primitive_map.functions.iter() {
            // Currently we are not creating 1 per function source and target.
            // We reuse the topics that were created from the data models.
            // Unless for Flow migrations where we will have to create new topics.

            if function.is_flow_migration() {
                let (source_topic, target_topic) = Topic::from_migration_function(function);

                let function_process =
                    FunctionProcess::from_migration_functon(function, &source_topic, &target_topic);

                topics.insert(source_topic.id(), source_topic);
                topics.insert(target_topic.id(), target_topic);
                function_processes.insert(function_process.id(), function_process);
            } else {
                let topics: Vec<String> = topics.values().map(|t| t.id()).collect();

                let function_process = FunctionProcess::from_functon(function, &topics);
                function_processes.insert(function_process.id(), function_process);
            }
        }

        InfrastructureMap {
            // primitive_map,
            topics,
            api_endpoints,
            topic_to_table_sync_processes,
            tables,
            function_processes,
        }
    }

    // The current implementation is simple in which it goes over some of the
    // changes several times. Something could be done to optimize this.
    // There is probably a way to make this also more generic so that we don't have very
    // similar code for each type of change.
    pub fn diff(&self, target_map: &InfrastructureMap) -> InfraChanges {
        let mut changes = InfraChanges::default();

        // =================================================================
        //                              Topics
        // =================================================================

        for (id, topic) in &self.topics {
            if let Some(target_topic) = target_map.topics.get(id) {
                if topic != target_topic {
                    changes
                        .streaming_engine_changes
                        .push(StreamingChange::Topic(Change::<Topic>::Updated {
                            before: topic.clone(),
                            after: target_topic.clone(),
                        }));
                }
            } else {
                changes
                    .streaming_engine_changes
                    .push(StreamingChange::Topic(Change::<Topic>::Removed(
                        topic.clone(),
                    )));
            }
        }

        for (id, topic) in &target_map.topics {
            if !self.topics.contains_key(id) {
                changes
                    .streaming_engine_changes
                    .push(StreamingChange::Topic(Change::<Topic>::Added(
                        topic.clone(),
                    )));
            }
        }

        // =================================================================
        //                              API Endpoints
        // =================================================================

        for (id, api_endpoint) in &self.api_endpoints {
            if let Some(target_api_endpoint) = target_map.api_endpoints.get(id) {
                if api_endpoint != target_api_endpoint {
                    changes.api_changes.push(ApiChange::ApiEndpoint(
                        Change::<ApiEndpoint>::Updated {
                            before: api_endpoint.clone(),
                            after: target_api_endpoint.clone(),
                        },
                    ));
                }
            } else {
                changes
                    .api_changes
                    .push(ApiChange::ApiEndpoint(Change::<ApiEndpoint>::Removed(
                        api_endpoint.clone(),
                    )));
            }
        }

        for (id, api_endpoint) in &target_map.api_endpoints {
            if !self.api_endpoints.contains_key(id) {
                changes
                    .api_changes
                    .push(ApiChange::ApiEndpoint(Change::<ApiEndpoint>::Added(
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
                    changes
                        .olap_changes
                        .push(OlapChange::Table(Change::<Table>::Updated {
                            before: table.clone(),
                            after: target_table.clone(),
                        }));
                }
            } else {
                changes
                    .olap_changes
                    .push(OlapChange::Table(Change::<Table>::Removed(table.clone())));
            }
        }

        for (id, table) in &target_map.tables {
            if !self.tables.contains_key(id) {
                changes
                    .olap_changes
                    .push(OlapChange::Table(Change::<Table>::Added(table.clone())));
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
                    changes
                        .processes_changes
                        .push(ProcessChange::TopicToTableSyncProcess(Change::<
                            TopicToTableSyncProcess,
                        >::Updated {
                            before: topic_to_table_sync_process.clone(),
                            after: target_topic_to_table_sync_process.clone(),
                        }));
                }
            } else {
                changes
                    .processes_changes
                    .push(ProcessChange::TopicToTableSyncProcess(Change::<
                        TopicToTableSyncProcess,
                    >::Removed(
                        topic_to_table_sync_process.clone(),
                    )));
            }
        }

        for (id, topic_to_table_sync_process) in &target_map.topic_to_table_sync_processes {
            if !self.topic_to_table_sync_processes.contains_key(id) {
                changes
                    .processes_changes
                    .push(ProcessChange::TopicToTableSyncProcess(Change::<
                        TopicToTableSyncProcess,
                    >::Added(
                        topic_to_table_sync_process.clone(),
                    )));
            }
        }

        // =================================================================
        //                             Function Processes
        // =================================================================

        for (id, function_process) in &self.function_processes {
            if let Some(target_function_process) = target_map.function_processes.get(id) {
                // In this case we don't do a comparison check because the function process is not just
                // dependendant on changing one file, but also on its dependencies. Untill we are able to
                // properly compare the function processes wholisticaly (File + Dependencies), we will just
                // assume that the function process has changed.
                changes
                    .processes_changes
                    .push(ProcessChange::FunctionProcess(
                        Change::<FunctionProcess>::Updated {
                            before: function_process.clone(),
                            after: target_function_process.clone(),
                        },
                    ));
            } else {
                changes
                    .processes_changes
                    .push(ProcessChange::FunctionProcess(
                        Change::<FunctionProcess>::Removed(function_process.clone()),
                    ));
            }
        }

        for (id, function_process) in &target_map.function_processes {
            if !self.function_processes.contains_key(id) {
                changes
                    .processes_changes
                    .push(ProcessChange::FunctionProcess(
                        Change::<FunctionProcess>::Added(function_process.clone()),
                    ));
            }
        }

        changes
    }

    /**
     * Generates all the changes that need to be made to the infrastructure in the case of the
     * infrastructure not having anything in place. ie everything needs to be created.
     */
    pub fn init(&self) -> InfraChanges {
        let olap_changes = self.init_tables();
        let processes_changes = self.init_processes();
        let api_changes = self.init_api_endpoints();
        let streaming_engine_changes = self.init_topics();

        InfraChanges {
            olap_changes,
            processes_changes,
            api_changes,
            streaming_engine_changes,
        }
    }

    pub fn init_topics(&self) -> Vec<StreamingChange> {
        self.topics
            .values()
            .map(|topic| StreamingChange::Topic(Change::<Topic>::Added(topic.clone())))
            .collect()
    }

    pub fn init_api_endpoints(&self) -> Vec<ApiChange> {
        self.api_endpoints
            .values()
            .map(|api_endpoint| {
                ApiChange::ApiEndpoint(Change::<ApiEndpoint>::Added(api_endpoint.clone()))
            })
            .collect()
    }

    pub fn init_tables(&self) -> Vec<OlapChange> {
        self.tables
            .values()
            .map(|table| OlapChange::Table(Change::<Table>::Added(table.clone())))
            .collect()
    }

    pub fn init_processes(&self) -> Vec<ProcessChange> {
        let mut topic_to_table_process_changes: Vec<ProcessChange> = self
            .topic_to_table_sync_processes
            .values()
            .map(|topic_to_table_sync_process| {
                ProcessChange::TopicToTableSyncProcess(Change::<TopicToTableSyncProcess>::Added(
                    topic_to_table_sync_process.clone(),
                ))
            })
            .collect();

        let mut function_process_changes: Vec<ProcessChange> = self
            .function_processes
            .values()
            .map(|function_process| {
                ProcessChange::FunctionProcess(Change::<FunctionProcess>::Added(
                    function_process.clone(),
                ))
            })
            .collect();

        topic_to_table_process_changes.append(&mut function_process_changes);

        topic_to_table_process_changes
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::{
        framework::{
            core::primitive_map::PrimitiveMap, data_model::model::DataModel,
            languages::SupportedLanguages,
        },
        project::Project,
    };

    #[tokio::test]
    #[ignore]
    async fn test_infra_map() {
        let project = Project::new(
            Path::new("/Users/nicolas/code/514/test"),
            "test".to_string(),
            SupportedLanguages::Typescript,
        );
        let primitive_map = PrimitiveMap::load(&project).await;
        println!("{:?}", primitive_map);
        assert!(primitive_map.is_ok());

        let infra_map = super::InfrastructureMap::new(primitive_map.unwrap());
        println!("{:?}", infra_map);
    }

    #[tokio::test]
    #[ignore]
    async fn test_infra_diff_map() {
        let project = Project::new(
            Path::new("/Users/nicolas/code/514/test"),
            "test".to_string(),
            SupportedLanguages::Typescript,
        );
        let primitive_map = PrimitiveMap::load(&project).await.unwrap();
        let mut new_target_primitive_map = primitive_map.clone();

        let data_model_name = "test";
        let data_model_version = "1.0.0";

        let new_data_model = DataModel {
            name: data_model_name.to_string(),
            version: data_model_version.to_string(),
            config: Default::default(),
            columns: vec![],
            abs_file_path: PathBuf::new(),
        };
        // Making some changes to the map
        new_target_primitive_map.datamodels.add(new_data_model);

        new_target_primitive_map
            .datamodels
            .remove(data_model_name, data_model_version)
            .unwrap();

        println!("Base Primitve Map: {:?} \n", primitive_map);
        println!("Target Primitive Map {:?} \n", new_target_primitive_map);

        let infra_map = super::InfrastructureMap::new(primitive_map);
        let new_infra_map = super::InfrastructureMap::new(new_target_primitive_map);

        let diffs = infra_map.diff(&new_infra_map);

        print!("Diffs: {:?}", diffs);
    }
}
