//! # Migration Plan Module
//!
//! This module provides data structures and functionality for migration plan generation,
//! storage, and execution. It enables the review and approval workflow for database
//! schema changes.
//!
//! ## Core Components
//!
//! - `MigrationPlan`: The main structure containing all information about a planned migration
//! - `PlanMetadata`: Additional metadata about the plan generation context
//! - `ExecutionState`: Tracks the current execution status of a migration plan
//!
//! ## Usage Flow
//!
//! 1. Generate a `MigrationPlan` from an `InfraPlan`
//! 2. Save the plan to a file for user review
//! 3. Load and execute the approved plan
//! 4. Track execution progress and handle failures

use crate::framework::core::infrastructure_map::InfrastructureMap;
use crate::framework::core::plan::InfraPlan;
use crate::infrastructure::olap::clickhouse::SerializableOlapOperation;
use crate::infrastructure::olap::ddl_ordering::order_olap_changes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A comprehensive migration plan that can be reviewed, approved, and executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// Timestamp when this plan was generated
    pub created_at: DateTime<Utc>,
    /// JSON representation of the source infra map planned against
    pub source_infra_map: String,
    /// DB Operations to run
    pub operations: Vec<SerializableOlapOperation>,
}

impl MigrationPlan {
    /// Creates a new migration plan from an infrastructure plan
    pub fn from_infra_plan(
        infra_plan: &InfraPlan,
        source_infra_map: InfrastructureMap,
    ) -> Result<Self, MigrationPlanError> {
        // Convert OLAP changes to atomic operations
        let (teardown_ops, setup_ops) = order_olap_changes(&infra_plan.changes.olap_changes)?;

        // Combine teardown and setup operations into a single vector
        // Teardown operations are executed first, then setup operations
        let mut operations = Vec::new();

        // Add teardown operations first
        for op in teardown_ops {
            operations.push(op.operation.to_minimal());
        }

        // Add setup operations second
        for op in setup_ops {
            operations.push(op.operation.to_minimal());
        }

        Ok(MigrationPlan {
            created_at: Utc::now(),
            source_infra_map: serde_json::to_string(&source_infra_map)?,
            operations,
        })
    }

    /// Returns the total number of operations
    pub fn total_operations(&self) -> usize {
        self.operations.len()
    }
}

/// Errors that can occur during migration plan operations
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MigrationPlanError {
    /// Error during plan ordering
    #[error("Failed to order OLAP operations")]
    PlanOrdering(#[from] crate::infrastructure::olap::ddl_ordering::PlanOrderingError),

    /// Serialization/deserialization error
    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),
}
