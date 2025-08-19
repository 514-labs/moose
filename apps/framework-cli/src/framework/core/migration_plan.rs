use crate::framework::core::infrastructure_map::{InfraChanges, InfrastructureMap};
use crate::infrastructure::olap::clickhouse::SerializableOlapOperation;
use crate::infrastructure::olap::ddl_ordering::{order_olap_changes, PlanOrderingError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A comprehensive migration plan that can be reviewed, approved, and executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// Timestamp when this plan was generated
    pub created_at: DateTime<Utc>,
    /// DB Operations to run
    pub operations: Vec<SerializableOlapOperation>,
}

pub struct MigrationPlanWithBeforeAfter {
    pub remote_state: InfrastructureMap,
    pub local_infra_map: InfrastructureMap,
    pub db_migration: MigrationPlan,
}

pub const MIGRATION_SCHEMA: &'static str =
    include_str!("../../utilities/migration_plan_schema.json");

impl MigrationPlan {
    /// Creates a new migration plan from an infrastructure plan
    pub fn from_infra_plan(infra_plan_changes: &InfraChanges) -> Result<Self, PlanOrderingError> {
        // Convert OLAP changes to atomic operations
        let (teardown_ops, setup_ops) = order_olap_changes(&infra_plan_changes.olap_changes)?;

        // Combine teardown and setup operations into a single vector
        // Teardown operations are executed first, then setup operations
        let mut operations = Vec::new();

        // Add teardown operations first
        for op in teardown_ops {
            operations.push(op.to_minimal());
        }

        // Add setup operations second
        for op in setup_ops {
            operations.push(op.to_minimal());
        }

        Ok(MigrationPlan {
            created_at: Utc::now(),
            operations,
        })
    }

    /// Returns the total number of operations
    pub fn total_operations(&self) -> usize {
        self.operations.len()
    }
}
