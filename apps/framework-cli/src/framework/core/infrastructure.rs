use serde::{Deserialize, Serialize};

pub mod api_endpoint;
pub mod function_process;
pub mod table;
pub mod topic;
pub mod topic_to_table_sync_process;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub enum InfrastructureSignature {
    Table { id: String },
    Topic { id: String },
    ApiEndpoint { id: String },
    TopicToTableSyncProcess { id: String },
}

pub trait DataLineage {
    fn receives_data_from(&self) -> Vec<InfrastructureSignature>;

    fn pulls_data_from(&self) -> Vec<InfrastructureSignature>;

    fn pushes_data_to(&self) -> Vec<InfrastructureSignature>;
}
