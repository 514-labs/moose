use crate::proto::infrastructure_map::SqlResource as ProtoSqlResource;
use serde::{Deserialize, Serialize};

use super::DataLineage;
use super::InfrastructureSignature;

/// Represents a SQL resource defined within the infrastructure configuration.
///
/// This struct holds information about a SQL resource, including its name,
/// setup and teardown scripts, and its data lineage relationships with other
/// infrastructure components.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SqlResource {
    /// The unique name identifier for the SQL resource.
    pub name: String,

    /// A list of SQL commands or script paths executed during the setup phase.
    pub setup: Vec<String>,
    /// A list of SQL commands or script paths executed during the teardown phase.
    pub teardown: Vec<String>,

    /// Signatures of infrastructure components from which this SQL resource pulls data.
    #[serde(alias = "pullsDataFrom")]
    pub pulls_data_from: Vec<InfrastructureSignature>,
    /// Signatures of infrastructure components to which this SQL resource pushes data.
    #[serde(alias = "pushesDataTo")]
    pub pushes_data_to: Vec<InfrastructureSignature>,
}

impl SqlResource {
    /// Converts the `SqlResource` struct into its corresponding Protobuf representation.
    pub fn to_proto(&self) -> ProtoSqlResource {
        ProtoSqlResource {
            name: self.name.clone(),
            setup: self.setup.clone(),
            teardown: self.teardown.clone(),
            special_fields: Default::default(),
            pulls_data_from: self.pulls_data_from.iter().map(|s| s.to_proto()).collect(),
            pushes_data_to: self.pushes_data_to.iter().map(|s| s.to_proto()).collect(),
        }
    }

    /// Creates a `SqlResource` struct from its Protobuf representation.
    pub fn from_proto(proto: ProtoSqlResource) -> Self {
        Self {
            name: proto.name,
            setup: proto.setup,
            teardown: proto.teardown,
            pulls_data_from: proto
                .pulls_data_from
                .into_iter()
                .map(InfrastructureSignature::from_proto)
                .collect(),
            pushes_data_to: proto
                .pushes_data_to
                .into_iter()
                .map(InfrastructureSignature::from_proto)
                .collect(),
        }
    }
}

/// Implements the `DataLineage` trait for `SqlResource`.
///
/// This allows querying the data flow relationships of the SQL resource.
impl DataLineage for SqlResource {
    /// Returns the signatures of infrastructure components from which this resource pulls data.
    fn pulls_data_from(&self) -> Vec<InfrastructureSignature> {
        self.pulls_data_from.clone()
    }

    /// Returns the signatures of infrastructure components to which this resource pushes data.
    fn pushes_data_to(&self) -> Vec<InfrastructureSignature> {
        self.pushes_data_to.clone()
    }
}
