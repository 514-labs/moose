//! # Plan Generation Routine
//!
//! This module provides the CLI routine for generating migration plans.
//! It analyzes the current project state and generates a comprehensive
//! migration plan that can be reviewed and approved before execution.

use std::fs;
use std::path::Path;

use log::info;

use crate::cli::display::Message;
use crate::cli::routines::{RoutineFailure, RoutineSuccess};
use crate::framework::core::migration_plan::MigrationPlan;
use crate::framework::core::plan::plan_changes;
use crate::infrastructure::redis::redis_client::RedisClient;
use crate::project::Project;
use crate::utilities::constants::DEFAULT_PLAN_PATH;

/// Generates a migration plan and saves it to a file for review
pub async fn generate_migration_plan(
    project: &Project,
    redis_client: &RedisClient,
    force: bool,
) -> Result<RoutineSuccess, RoutineFailure> {
    info!("Generating migration plan...");

    let plan_path: &Path = DEFAULT_PLAN_PATH.as_ref();

    if plan_path.exists() && !force {
        return Err(RoutineFailure::error(Message::new(
            "Plan".to_string(),
            format!(
                "Plan file already exists: {:?}. Use --force to overwrite.",
                plan_path
            ),
        )));
    }

    project.internal_dir().map_err(|e| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to validate internal directory.".to_string(),
            ),
            e,
        )
    })?;

    let (source, infra_plan) = plan_changes(redis_client, project).await.map_err(|e| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to plan against current state.".to_string(),
            ),
            e,
        )
    })?;
    let migration_plan = MigrationPlan::from_infra_plan(&infra_plan, source).map_err(|e| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to calculate DB migration plan.".to_string(),
            ),
            e,
        )
    })?;

    // Save the plan to file
    let plan_yaml = serde_yaml::to_string(&migration_plan).map_err(|e| {
        RoutineFailure::new(
            Message::new(
                "Migration".to_string(),
                "plan serialization failed.".to_string(),
            ),
            e,
        )
    })?;

    fs::write(plan_path, plan_yaml).map_err(|e| {
        RoutineFailure::new(
            Message::new("Migration".to_string(), "plan writing failed.".to_string()),
            e,
        )
    })?;

    // Display summary
    info!("Migration plan generated successfully!");
    info!("Plan saved to: {}", plan_path.display());
    info!("Total operations: {}", migration_plan.total_operations());

    let message = format!(
        "Migration plan saved to {} with {} operations",
        plan_path.display(),
        migration_plan.total_operations(),
    );

    Ok(RoutineSuccess::success(Message::new(
        "Plan Generate".to_string(),
        message,
    )))
}
