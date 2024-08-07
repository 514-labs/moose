use super::infrastructure::api_endpoint::ApiEndpoint;
use super::infrastructure::consumption_webserver::ConsumptionApiWebServer;
use super::infrastructure::function_process::FunctionProcess;
use super::infrastructure::olap_process::OlapProcess;
use super::infrastructure::table::Table;
use super::infrastructure::topic::Topic;
use super::infrastructure::topic_to_table_sync_process::TopicToTableSyncProcess;
use super::infrastructure::view::View;
use super::primitive_map::PrimitiveMap;
use crate::framework::controller::{InitialDataLoad, InitialDataLoadStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    View(Change<View>),
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
    OlapProcess(Change<OlapProcess>),
    ConsumptionApiWebServer(Change<ConsumptionApiWebServer>),
}

#[derive(Debug, Clone, Default)]
pub struct InfraChanges {
    pub olap_changes: Vec<OlapChange>,
    pub processes_changes: Vec<ProcessChange>,
    pub api_changes: Vec<ApiChange>,
    pub streaming_engine_changes: Vec<StreamingChange>,
    pub initial_data_loads: Vec<InitialDataLoadChange>,
}

#[derive(Debug, Clone)]
pub enum InitialDataLoadChange {
    Addition(InitialDataLoad),
    Resumption {
        load: InitialDataLoad,
        resume_from: i64,
    },
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
    pub views: HashMap<String, View>,

    pub topic_to_table_sync_processes: HashMap<String, TopicToTableSyncProcess>,
    pub function_processes: HashMap<String, FunctionProcess>,

    // TODO change to a hashmap of processes when we have several
    pub block_db_processes: OlapProcess,

    // Not sure if we will want to change that or not in the future to be able to tell
    // the new consumption endpoints that were added or removed.
    pub consumption_api_web_server: ConsumptionApiWebServer,

    pub initial_data_loads: HashMap<String, InitialDataLoad>,
}

impl InfrastructureMap {
    pub fn new(primitive_map: PrimitiveMap) -> InfrastructureMap {
        let mut tables = HashMap::new();
        let mut views = HashMap::new();
        let mut topics = HashMap::new();
        let mut api_endpoints = HashMap::new();
        let mut topic_to_table_sync_processes = HashMap::new();
        let mut function_processes = HashMap::new();
        let mut initial_data_loads = HashMap::new();

        let mut data_models_that_have_not_changed_with_new_version = Vec::new();

        for data_model in primitive_map.data_models_iter() {
            if primitive_map
                .datamodels
                .has_data_model_changed_with_previous_version(&data_model.name, &data_model.version)
            {
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
            } else {
                // We wait to have processed all the datamodels to process the ones that don't have changes
                // That way we can refer to infrastructure that was created by those older versions.
                data_models_that_have_not_changed_with_new_version.push(data_model);
            }
        }

        // We process the data models that have not changed with their registered versions.
        // For the ones that require storage, we have views that points to the oldest table that has the data
        // with the same schema. We also reused the same topic that was created for the previous version.
        for data_model in data_models_that_have_not_changed_with_new_version {
            match primitive_map
                .datamodels
                .find_earliest_similar_version(&data_model.name, &data_model.version)
            {
                Some(previous_version_model) => {
                    // This will be already created with the previous data model.
                    // That's why we don't add it to the map
                    let previous_version_topic = Topic::from_data_model(previous_version_model);
                    let api_endpoint =
                        ApiEndpoint::from_data_model(data_model, &previous_version_topic);

                    if data_model.config.storage.enabled
                        && previous_version_model.config.storage.enabled
                    {
                        let view = View::alias_view(data_model, previous_version_model);
                        views.insert(view.id(), view);
                    }

                    api_endpoints.insert(api_endpoint.id(), api_endpoint);
                }
                None => {
                    log::error!(
                        "Could not find previous version with no change for data model: {} {}",
                        data_model.name,
                        data_model.version
                    );
                    log::debug!("Data Models Dump: {:?}", primitive_map.datamodels);
                }
            }
        }

        for function in primitive_map.functions.iter() {
            // Currently we are not creating 1 per function source and target.
            // We reuse the topics that were created from the data models.
            // Unless for streaming function migrations where we will have to create new topics.

            if function.is_migration() {
                let (source_topic, target_topic) = Topic::from_migration_function(function);

                let function_process = FunctionProcess::from_migration_function(
                    function,
                    &source_topic,
                    &target_topic,
                );

                initial_data_loads.insert(
                    function_process.id(),
                    InitialDataLoad {
                        table: function.source_data_model.to_table(),
                        topic: source_topic.name.clone(),
                        // it doesn't mean it is completed, it means we want it to be completed
                        status: InitialDataLoadStatus::Completed,
                    },
                );
                topics.insert(source_topic.id(), source_topic);
                topics.insert(target_topic.id(), target_topic);

                function_processes.insert(function_process.id(), function_process);
            } else {
                let topics: Vec<String> = topics.values().map(|t| t.id()).collect();

                let function_process = FunctionProcess::from_function(function, &topics);
                function_processes.insert(function_process.id(), function_process);
            }
        }

        // TODO update here when we have several aggregation processes
        let block_db_processes = OlapProcess::from_aggregation(&primitive_map.aggregation);

        // We are currently not
        let consumption_api_web_server = ConsumptionApiWebServer {};

        InfrastructureMap {
            // primitive_map,
            topics,
            api_endpoints,
            topic_to_table_sync_processes,
            tables,
            views,
            function_processes,
            block_db_processes,
            consumption_api_web_server,
            initial_data_loads,
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
        //                              Views
        // =================================================================

        for (id, view) in &self.views {
            if let Some(target_view) = target_map.views.get(id) {
                if view != target_view {
                    changes
                        .olap_changes
                        .push(OlapChange::View(Change::<View>::Updated {
                            before: view.clone(),
                            after: target_view.clone(),
                        }));
                }
            } else {
                changes
                    .olap_changes
                    .push(OlapChange::View(Change::<View>::Removed(view.clone())));
            }
        }

        for (id, view) in &target_map.views {
            if !self.tables.contains_key(id) {
                changes
                    .olap_changes
                    .push(OlapChange::View(Change::<View>::Added(view.clone())));
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

        // =================================================================
        //                             Initial Data Loads
        // =================================================================
        for (id, load) in &target_map.initial_data_loads {
            let existing = self.initial_data_loads.get(id);
            match existing {
                None => changes
                    .initial_data_loads
                    .push(InitialDataLoadChange::Addition(load.clone())),
                Some(existing) => {
                    match existing.status {
                        InitialDataLoadStatus::InProgress(resume_from) => changes
                            .initial_data_loads
                            .push(InitialDataLoadChange::Resumption {
                                resume_from,
                                load: load.clone(),
                            }),
                        // nothing to do
                        InitialDataLoadStatus::Completed => {}
                    }
                }
            }
        }

        // =================================================================
        //                             Aggregation Processes
        // =================================================================

        // Until we refactor to have multiple processes, we will consider that we need to restart
        // the process all the times and the aggregations changes all the time.
        // Once we do the other refactor, we will be able to compare the changes and only restart
        // the process if there are changes

        // currently we assume there is always a change and restart the processes
        changes.processes_changes.push(ProcessChange::OlapProcess(
            Change::<OlapProcess>::Updated {
                before: OlapProcess {},
                after: OlapProcess {},
            },
        ));

        // =================================================================
        //                          Consumption Process
        // =================================================================

        // We are currently not tracking individual consumption endpoints, so we will just restart
        // the consumption web server when something changed. we might want to change that in the future
        // to be able to only make changes when something in the dependency tree of a consumption api has
        // changed.

        changes
            .processes_changes
            .push(ProcessChange::ConsumptionApiWebServer(Change::<
                ConsumptionApiWebServer,
            >::Updated {
                before: ConsumptionApiWebServer {},
                after: ConsumptionApiWebServer {},
            }));

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
        let initial_data_loads = self.init_data_loads();

        InfraChanges {
            olap_changes,
            processes_changes,
            api_changes,
            streaming_engine_changes,
            initial_data_loads,
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

        // TODO Change this when we have multiple processes for aggregations
        topic_to_table_process_changes.push(ProcessChange::OlapProcess(
            Change::<OlapProcess>::Added(OlapProcess {}),
        ));

        topic_to_table_process_changes.push(ProcessChange::ConsumptionApiWebServer(Change::<
            ConsumptionApiWebServer,
        >::Added(
            ConsumptionApiWebServer {},
        )));

        topic_to_table_process_changes
    }

    pub fn init_data_loads(&self) -> Vec<InitialDataLoadChange> {
        self.initial_data_loads
            .values()
            .map(|load| {
                InitialDataLoadChange::Addition(InitialDataLoad {
                    // if existing deployment is empty, there is no initial data to load
                    status: InitialDataLoadStatus::Completed,
                    ..load.clone()
                })
            })
            .collect()
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
            .remove(data_model_name, data_model_version);

        println!("Base Primitive Map: {:?} \n", primitive_map);
        println!("Target Primitive Map {:?} \n", new_target_primitive_map);

        let infra_map = super::InfrastructureMap::new(primitive_map);
        let new_infra_map = super::InfrastructureMap::new(new_target_primitive_map);

        let diffs = infra_map.diff(&new_infra_map);

        print!("Diffs: {:?}", diffs);
    }
}
