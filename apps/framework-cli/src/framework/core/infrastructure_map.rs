use super::infrastructure::api_endpoint::{APIType, ApiEndpoint};
use super::infrastructure::consumption_webserver::ConsumptionApiWebServer;
use super::infrastructure::function_process::FunctionProcess;
use super::infrastructure::olap_process::OlapProcess;
use super::infrastructure::table::{Column, Table};
use super::infrastructure::topic::Topic;
use super::infrastructure::topic_sync_process::{TopicToTableSyncProcess, TopicToTopicSyncProcess};
use super::infrastructure::view::View;
use super::primitive_map::PrimitiveMap;
use crate::framework::controller::{InitialDataLoad, InitialDataLoadStatus};
use crate::infrastructure::redis::redis_client::RedisClient;
use crate::infrastructure::stream::redpanda::RedpandaConfig;
use crate::proto::infrastructure_map::InfrastructureMap as ProtoInfrastructureMap;
use anyhow::Result;
use protobuf::{EnumOrUnknown, Message};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

impl PrimitiveSignature {
    pub fn to_proto(&self) -> crate::proto::infrastructure_map::PrimitiveSignature {
        crate::proto::infrastructure_map::PrimitiveSignature {
            name: self.name.clone(),
            primitive_type: EnumOrUnknown::new(self.primitive_type.to_proto()),
            special_fields: Default::default(),
        }
    }
}

impl PrimitiveTypes {
    fn to_proto(&self) -> crate::proto::infrastructure_map::PrimitiveTypes {
        match self {
            PrimitiveTypes::DataModel => {
                crate::proto::infrastructure_map::PrimitiveTypes::DATA_MODEL
            }
            PrimitiveTypes::Function => crate::proto::infrastructure_map::PrimitiveTypes::FUNCTION,
            PrimitiveTypes::DBBlock => crate::proto::infrastructure_map::PrimitiveTypes::DB_BLOCK,
            PrimitiveTypes::ConsumptionAPI => {
                crate::proto::infrastructure_map::PrimitiveTypes::CONSUMPTION_API
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ColumnChange {
    Added(Column),
    Removed(Column),
    Updated { before: Column, after: Column },
}

#[derive(Debug, Clone)]
pub struct OrderByChange {
    pub before: Vec<String>,
    pub after: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum TableChange {
    Added(Table),
    Removed(Table),
    Updated {
        name: String,
        column_changes: Vec<ColumnChange>,
        order_by_change: OrderByChange,
        before: Table,
        after: Table,
    },
}

#[derive(Debug, Clone)]
pub enum Change<T> {
    Added(Box<T>),
    Removed(Box<T>),
    Updated { before: Box<T>, after: Box<T> },
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
#[allow(clippy::large_enum_variant)]
pub enum OlapChange {
    Table(TableChange),
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
    TopicToTopicSyncProcess(Change<TopicToTopicSyncProcess>),
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
    #[serde(default = "HashMap::new")]
    pub topic_to_topic_sync_processes: HashMap<String, TopicToTopicSyncProcess>,
    pub function_processes: HashMap<String, FunctionProcess>,

    // TODO change to a hashmap of processes when we have several
    pub block_db_processes: OlapProcess,

    // Not sure if we will want to change that or not in the future to be able to tell
    // the new consumption endpoints that were added or removed.
    pub consumption_api_web_server: ConsumptionApiWebServer,

    pub initial_data_loads: HashMap<String, InitialDataLoad>,
}

impl InfrastructureMap {
    pub fn diff_tables(
        self_tables: &HashMap<String, Table>,
        target_tables: &HashMap<String, Table>,
        olap_changes: &mut Vec<OlapChange>,
    ) {
        for (id, table) in self_tables {
            if let Some(target_table) = target_tables.get(id) {
                if table != target_table {
                    let column_changes = compute_table_diff(table, target_table);
                    olap_changes.push(OlapChange::Table(TableChange::Updated {
                        name: table.name.clone(),
                        column_changes,
                        order_by_change: OrderByChange {
                            before: table.order_by.clone(),
                            after: target_table.order_by.clone(),
                        },
                        before: table.clone(),
                        after: target_table.clone(),
                    }));
                }
            } else {
                olap_changes.push(OlapChange::Table(TableChange::Removed(table.clone())));
            }
        }

        for (id, table) in target_tables {
            if !self_tables.contains_key(id) {
                olap_changes.push(OlapChange::Table(TableChange::Added(table.clone())));
            }
        }
    }

    // we want to add the prefix as soon as possible
    // to avoid worrying whether the topic name is prefixed
    // we dump the infra map during build, when the prefix is unavailable
    // so the prefix adding has to be written as a mutation
    //
    // an alternative is to dump the PrimitiveMap during build,
    // and add `RedpandaConfig` param to the `new(PrimitiveMap)->InfrastructureMap` function.
    pub fn with_topic_namespace(&mut self, redpanda_config: &RedpandaConfig) {
        let topics = std::mem::take(&mut self.topics);
        for mut topic in topics.into_values() {
            topic.name = redpanda_config.prefix_with_namespace(&topic.name);
            self.topics.insert(topic.id(), topic);
        }

        let api_endpoints = std::mem::take(&mut self.api_endpoints);
        for mut api_endpoint in api_endpoints.into_values() {
            match api_endpoint.api_type {
                APIType::INGRESS {
                    ref mut target_topic,
                    ..
                } => *target_topic = redpanda_config.prefix_with_namespace(target_topic),
                APIType::EGRESS => {}
            }
            self.api_endpoints.insert(api_endpoint.id(), api_endpoint);
        }

        let topic_to_table_sync_processes = std::mem::take(&mut self.topic_to_table_sync_processes);
        for mut process in topic_to_table_sync_processes.into_values() {
            process.source_topic_id =
                redpanda_config.prefix_with_namespace(&process.source_topic_id);
            self.topic_to_table_sync_processes
                .insert(process.id(), process);
        }

        let topic_to_topic_sync_processes = std::mem::take(&mut self.topic_to_topic_sync_processes);
        for mut process in topic_to_topic_sync_processes.into_values() {
            process.source_topic_id =
                redpanda_config.prefix_with_namespace(&process.source_topic_id);
            process.target_topic_id =
                redpanda_config.prefix_with_namespace(&process.target_topic_id);

            self.topic_to_topic_sync_processes
                .insert(process.id(), process);
        }

        let function_processes = std::mem::take(&mut self.function_processes);
        for mut process in function_processes.into_values() {
            process.source_topic = redpanda_config.prefix_with_namespace(&process.source_topic);
            if !process.target_topic.is_empty() {
                process.target_topic = redpanda_config.prefix_with_namespace(&process.target_topic);
            }
            self.function_processes.insert(process.id(), process);
        }
        let initial_data_loads = std::mem::take(&mut self.initial_data_loads);
        for (id, mut process) in initial_data_loads.into_iter() {
            process.topic = redpanda_config.prefix_with_namespace(&process.topic);
            self.initial_data_loads.insert(id, process);
        }
    }

    pub fn new(primitive_map: PrimitiveMap) -> InfrastructureMap {
        let mut tables = HashMap::new();
        let mut views = HashMap::new();
        let mut topics = HashMap::new();
        let mut api_endpoints = HashMap::new();
        let mut topic_to_table_sync_processes = HashMap::new();
        let mut topic_to_topic_sync_processes = HashMap::new();
        let mut function_processes = HashMap::new();
        let mut initial_data_loads = HashMap::new();

        let mut data_models_that_have_not_changed_with_new_version = Vec::new();

        for data_model in primitive_map.data_models_iter() {
            if primitive_map
                .datamodels
                .has_data_model_changed_with_previous_version(
                    &data_model.name,
                    data_model.version.as_str(),
                )
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
                .find_earliest_similar_version(&data_model.name, data_model.version.as_str())
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
                    &target_topic.clone().unwrap(),
                );

                let sync_process = TopicToTableSyncProcess::new(
                    &target_topic.clone().unwrap(),
                    &function.target_data_model.as_ref().unwrap().to_table(),
                );
                topic_to_table_sync_processes.insert(sync_process.id(), sync_process);

                let topic_sync = TopicToTopicSyncProcess::from_migration_function(function);
                topic_to_topic_sync_processes.insert(topic_sync.id(), topic_sync);

                initial_data_loads.insert(
                    function_process.id(),
                    InitialDataLoad {
                        table: function.source_data_model.to_table(),
                        topic: source_topic.name.clone(),
                        // it doesn't mean it is completed, it means the desired state is Completed
                        status: InitialDataLoadStatus::Completed,
                    },
                );
                topics.insert(source_topic.id(), source_topic);
                if let Some(target) = target_topic.clone() {
                    topics.insert(target.id(), target.clone());
                }

                function_processes.insert(function_process.id(), function_process);
            } else {
                let function_process = FunctionProcess::from_function(function, &topics);
                function_processes.insert(function_process.id(), function_process);
            }
        }

        // TODO update here when we have several aggregation processes
        let block_db_processes = OlapProcess::from_aggregation(&primitive_map.aggregation);

        // consumption api endpoints
        let consumption_api_web_server = ConsumptionApiWebServer {};
        for api_endpoint in primitive_map.consumption.endpoint_files {
            api_endpoints.insert(api_endpoint.id(), api_endpoint.into());
        }

        InfrastructureMap {
            // primitive_map,
            topics,
            api_endpoints,
            topic_to_table_sync_processes,
            topic_to_topic_sync_processes,
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
                            before: Box::new(topic.clone()),
                            after: Box::new(target_topic.clone()),
                        }));
                }
            } else {
                changes
                    .streaming_engine_changes
                    .push(StreamingChange::Topic(Change::<Topic>::Removed(Box::new(
                        topic.clone(),
                    ))));
            }
        }

        for (id, topic) in &target_map.topics {
            if !self.topics.contains_key(id) {
                changes
                    .streaming_engine_changes
                    .push(StreamingChange::Topic(Change::<Topic>::Added(Box::new(
                        topic.clone(),
                    ))));
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
                            before: Box::new(api_endpoint.clone()),
                            after: Box::new(target_api_endpoint.clone()),
                        },
                    ));
                }
            } else {
                changes
                    .api_changes
                    .push(ApiChange::ApiEndpoint(Change::<ApiEndpoint>::Removed(
                        Box::new(api_endpoint.clone()),
                    )));
            }
        }

        for (id, api_endpoint) in &target_map.api_endpoints {
            if !self.api_endpoints.contains_key(id) {
                changes
                    .api_changes
                    .push(ApiChange::ApiEndpoint(Change::<ApiEndpoint>::Added(
                        Box::new(api_endpoint.clone()),
                    )));
            }
        }

        // =================================================================
        //                              Tables
        // =================================================================

        Self::diff_tables(&self.tables, &target_map.tables, &mut changes.olap_changes);

        // =================================================================
        //                              Views
        // =================================================================

        for (id, view) in &self.views {
            if let Some(target_view) = target_map.views.get(id) {
                if view != target_view {
                    changes
                        .olap_changes
                        .push(OlapChange::View(Change::<View>::Updated {
                            before: Box::new(view.clone()),
                            after: Box::new(target_view.clone()),
                        }));
                }
            } else {
                changes
                    .olap_changes
                    .push(OlapChange::View(Change::<View>::Removed(Box::new(
                        view.clone(),
                    ))));
            }
        }

        for (id, view) in &target_map.views {
            if !self.views.contains_key(id) {
                changes
                    .olap_changes
                    .push(OlapChange::View(Change::<View>::Added(Box::new(
                        view.clone(),
                    ))));
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
                            before: Box::new(topic_to_table_sync_process.clone()),
                            after: Box::new(target_topic_to_table_sync_process.clone()),
                        }));
                }
            } else {
                changes
                    .processes_changes
                    .push(ProcessChange::TopicToTableSyncProcess(Change::<
                        TopicToTableSyncProcess,
                    >::Removed(
                        Box::new(topic_to_table_sync_process.clone()),
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
                        Box::new(topic_to_table_sync_process.clone()),
                    )));
            }
        }

        // =================================================================
        //                              Topic to Topic Sync Processes
        // =================================================================

        for (id, topic_to_topic_sync_process) in &self.topic_to_topic_sync_processes {
            if let Some(target_topic_to_topic_sync_process) =
                target_map.topic_to_topic_sync_processes.get(id)
            {
                if topic_to_topic_sync_process != target_topic_to_topic_sync_process {
                    changes
                        .processes_changes
                        .push(ProcessChange::TopicToTopicSyncProcess(Change::<
                            TopicToTopicSyncProcess,
                        >::Updated {
                            before: Box::new(topic_to_topic_sync_process.clone()),
                            after: Box::new(target_topic_to_topic_sync_process.clone()),
                        }));
                }
            } else {
                changes
                    .processes_changes
                    .push(ProcessChange::TopicToTopicSyncProcess(Change::<
                        TopicToTopicSyncProcess,
                    >::Removed(
                        Box::new(topic_to_topic_sync_process.clone()),
                    )));
            }
        }

        for (id, topic_to_topic_sync_process) in &target_map.topic_to_topic_sync_processes {
            if !self.topic_to_topic_sync_processes.contains_key(id) {
                changes
                    .processes_changes
                    .push(ProcessChange::TopicToTopicSyncProcess(Change::<
                        TopicToTopicSyncProcess,
                    >::Added(
                        Box::new(topic_to_topic_sync_process.clone()),
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
                            before: Box::new(function_process.clone()),
                            after: Box::new(target_function_process.clone()),
                        },
                    ));
            } else {
                changes
                    .processes_changes
                    .push(ProcessChange::FunctionProcess(
                        Change::<FunctionProcess>::Removed(Box::new(function_process.clone())),
                    ));
            }
        }

        for (id, function_process) in &target_map.function_processes {
            if !self.function_processes.contains_key(id) {
                changes
                    .processes_changes
                    .push(ProcessChange::FunctionProcess(
                        Change::<FunctionProcess>::Added(Box::new(function_process.clone())),
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
                before: Box::new(OlapProcess {}),
                after: Box::new(OlapProcess {}),
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
                before: Box::new(ConsumptionApiWebServer {}),
                after: Box::new(ConsumptionApiWebServer {}),
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
            .map(|topic| StreamingChange::Topic(Change::<Topic>::Added(Box::new(topic.clone()))))
            .collect()
    }

    pub fn init_api_endpoints(&self) -> Vec<ApiChange> {
        self.api_endpoints
            .values()
            .map(|api_endpoint| {
                ApiChange::ApiEndpoint(Change::<ApiEndpoint>::Added(Box::new(api_endpoint.clone())))
            })
            .collect()
    }

    pub fn init_tables(&self) -> Vec<OlapChange> {
        self.tables
            .values()
            .map(|table| OlapChange::Table(TableChange::Added(table.clone())))
            .collect()
    }

    pub fn init_processes(&self) -> Vec<ProcessChange> {
        let mut topic_to_table_process_changes: Vec<ProcessChange> = self
            .topic_to_table_sync_processes
            .values()
            .map(|topic_to_table_sync_process| {
                ProcessChange::TopicToTableSyncProcess(Change::<TopicToTableSyncProcess>::Added(
                    Box::new(topic_to_table_sync_process.clone()),
                ))
            })
            .collect();

        let mut topic_to_topic_process_changes: Vec<ProcessChange> = self
            .topic_to_topic_sync_processes
            .values()
            .map(|topic_to_table_sync_process| {
                ProcessChange::TopicToTopicSyncProcess(Change::<TopicToTopicSyncProcess>::Added(
                    Box::new(topic_to_table_sync_process.clone()),
                ))
            })
            .collect();
        topic_to_table_process_changes.append(&mut topic_to_topic_process_changes);

        let mut function_process_changes: Vec<ProcessChange> = self
            .function_processes
            .values()
            .map(|function_process| {
                ProcessChange::FunctionProcess(Change::<FunctionProcess>::Added(Box::new(
                    function_process.clone(),
                )))
            })
            .collect();

        topic_to_table_process_changes.append(&mut function_process_changes);

        // TODO Change this when we have multiple processes for aggregations
        topic_to_table_process_changes.push(ProcessChange::OlapProcess(
            Change::<OlapProcess>::Added(Box::new(OlapProcess {})),
        ));

        topic_to_table_process_changes.push(ProcessChange::ConsumptionApiWebServer(Change::<
            ConsumptionApiWebServer,
        >::Added(
            Box::new(ConsumptionApiWebServer {}),
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

    pub fn save_to_json(&self, path: &Path) -> Result<(), std::io::Error> {
        let json = serde_json::to_string(self)?;
        fs::write(path, json)
    }

    pub fn load_from_json(path: &Path) -> Result<Self, std::io::Error> {
        let json = fs::read_to_string(path)?;
        let infra_map = serde_json::from_str(&json)?;
        Ok(infra_map)
    }

    pub async fn store_in_redis(&self, redis_client: &RedisClient) -> Result<()> {
        use anyhow::Context;
        let encoded: Vec<u8> = self.to_proto().write_to_bytes()?;
        redis_client
            .set_with_service_prefix("infrastructure_map", &encoded)
            .await
            .context("Failed to store InfrastructureMap in Redis")?;

        Ok(())
    }

    pub fn to_proto(&self) -> ProtoInfrastructureMap {
        ProtoInfrastructureMap {
            topics: self
                .topics
                .iter()
                .map(|(k, v)| (k.clone(), v.to_proto()))
                .collect(),
            api_endpoints: self
                .api_endpoints
                .iter()
                .map(|(k, v)| (k.clone(), v.to_proto()))
                .collect(),
            tables: self
                .tables
                .iter()
                .map(|(k, v)| (k.clone(), v.to_proto()))
                .collect(),
            views: self
                .views
                .iter()
                .map(|(k, v)| (k.clone(), v.to_proto()))
                .collect(),
            topic_to_table_sync_processes: self
                .topic_to_table_sync_processes
                .iter()
                .map(|(k, v)| (k.clone(), v.to_proto()))
                .collect(),
            topic_to_topic_sync_processes: self
                .topic_to_topic_sync_processes
                .iter()
                .map(|(k, v)| (k.clone(), v.to_proto()))
                .collect(),
            function_processes: self
                .function_processes
                .iter()
                .map(|(k, v)| (k.clone(), v.to_proto()))
                .collect(),
            initial_data_loads: self
                .initial_data_loads
                .iter()
                .map(|(k, v)| (k.clone(), v.to_proto()))
                .collect(),
            special_fields: Default::default(),
        }
    }

    pub fn to_proto_bytes(&self) -> Vec<u8> {
        self.to_proto().write_to_bytes().unwrap()
    }
}

pub fn compute_table_diff(before: &Table, after: &Table) -> Vec<ColumnChange> {
    let mut diff = Vec::new();

    // Check for added or modified columns
    for after_col in &after.columns {
        match before.columns.iter().find(|c| c.name == after_col.name) {
            // If the column is in the before table, but different, then it is modified
            Some(before_col) if before_col != after_col => {
                diff.push(ColumnChange::Updated {
                    before: before_col.clone(),
                    after: after_col.clone(),
                });
            }
            // If the column is not in the before table, then it is added
            None => {
                diff.push(ColumnChange::Added(after_col.clone()));
            }
            _ => {}
        }
    }

    // Check for dropped columns
    for before_col in &before.columns {
        if !after.columns.iter().any(|c| c.name == before_col.name) {
            diff.push(ColumnChange::Removed(before_col.clone()));
        }
    }

    diff
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::framework::versions::Version;
    use crate::{
        framework::{
            core::{
                infrastructure::table::{Column, ColumnType, Table},
                infrastructure_map::{
                    compute_table_diff, ColumnChange, PrimitiveSignature, PrimitiveTypes,
                },
                primitive_map::PrimitiveMap,
            },
            data_model::model::DataModel,
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
            version: Version::from_string(data_model_version.to_string()),
            config: Default::default(),
            columns: vec![],
            abs_file_path: PathBuf::new(),
        };
        // Making some changes to the map
        new_target_primitive_map
            .datamodels
            .add(new_data_model)
            .unwrap();

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

    #[test]
    fn test_compute_table_diff() {
        let before = Table {
            name: "test_table".to_string(),
            deduplicate: false,
            columns: vec![
                Column {
                    name: "id".to_string(),
                    data_type: ColumnType::Int,
                    required: true,
                    unique: true,
                    primary_key: true,
                    default: None,
                },
                Column {
                    name: "name".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
                Column {
                    name: "to_be_removed".to_string(),
                    data_type: ColumnType::String,
                    required: false,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
            ],
            order_by: vec!["id".to_string()],
            version: Version::from_string("1.0".to_string()),
            source_primitive: PrimitiveSignature {
                name: "test_primitive".to_string(),
                primitive_type: PrimitiveTypes::DataModel,
            },
        };

        let after = Table {
            name: "test_table".to_string(),
            deduplicate: false,
            columns: vec![
                Column {
                    name: "id".to_string(),
                    data_type: ColumnType::BigInt, // Changed type
                    required: true,
                    unique: true,
                    primary_key: true,
                    default: None,
                },
                Column {
                    name: "name".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
                Column {
                    name: "age".to_string(), // New column
                    data_type: ColumnType::Int,
                    required: false,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
            ],
            order_by: vec!["id".to_string(), "name".to_string()], // Changed order_by
            version: Version::from_string("1.1".to_string()),
            source_primitive: PrimitiveSignature {
                name: "test_primitive".to_string(),
                primitive_type: PrimitiveTypes::DataModel,
            },
        };

        let diff = compute_table_diff(&before, &after);

        assert_eq!(diff.len(), 3);
        assert!(
            matches!(&diff[0], ColumnChange::Updated { before, after } if before.name == "id" && matches!(after.data_type, ColumnType::BigInt))
        );
        assert!(matches!(&diff[1], ColumnChange::Added(col) if col.name == "age"));
        assert!(matches!(&diff[2], ColumnChange::Removed(col) if col.name == "to_be_removed"));
    }
}
