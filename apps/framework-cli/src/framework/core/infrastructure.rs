//! Infrastructure module for framework core components.
//!
//! This module defines the core infrastructure components used throughout the framework,
//! providing abstractions for various data storage, processing, and communication mechanisms.
//! It establishes a consistent pattern for defining infrastructure components and their
//! relationships through data lineage.
//!
//! This is a platform agnostic module that can be used to define infrastructure components
//! for any platform. ie there should not be anything specific to a given warehouse or streaming engine
//! in this module.
//!
//! The infrastructure components are used to build the infrastructure map which is used to
//! generate the deployment plan.
//!
//! If components need to reference each other, they should do so by reference and not by value.

use serde::{Deserialize, Serialize};

pub mod api_endpoint;
pub mod consumption_webserver;
pub mod function_process;
pub mod olap_process;
pub mod orchestration_worker;
pub mod table;
pub mod topic;
pub mod topic_sync_process;
pub mod view;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

/// Represents the unique signature of an infrastructure component.
///
/// Infrastructure signatures are used to identify and reference various infrastructure
/// components throughout the system, enabling tracking of data flows and dependencies
/// between components.
///
/// Each variant corresponds to a specific type of infrastructure with a unique identifier.
pub enum InfrastructureSignature {
    /// Table storage infrastructure component
    Table { id: String },
    /// Messaging topic infrastructure component
    Topic { id: String },
    /// API endpoint infrastructure component
    ApiEndpoint { id: String },
    /// Process that synchronizes data from a topic to a table
    TopicToTableSyncProcess { id: String },
}

/// Defines the data flow relationships between infrastructure components.
///
/// This trait enables components to express their data lineage - how data flows into,
/// through, and out of the component. By implementing this trait, components can
/// participate in data flow analysis, dependency tracking, and observability.
///
/// The distinction between "receives", "pulls", and "pushes" represents different
/// data flow patterns:
/// - Receiving: Passive acceptance of data pushed by another component
/// - Pulling: Active fetching of data from another component
/// - Pushing: Active sending of data to another component
pub trait DataLineage {
    /// Returns infrastructure components that actively push data to this component.
    fn receives_data_from(&self) -> Vec<InfrastructureSignature>;

    /// Returns infrastructure components that this component actively pulls data from.
    fn pulls_data_from(&self) -> Vec<InfrastructureSignature>;

    /// Returns infrastructure components that this component actively pushes data to.
    fn pushes_data_to(&self) -> Vec<InfrastructureSignature>;
}
