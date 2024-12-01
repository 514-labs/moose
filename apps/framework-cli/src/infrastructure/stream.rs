use crate::{framework::core::infrastructure_map::StreamingChange, project::Project};

pub mod redpanda;
pub mod rpk;

#[derive(Debug, thiserror::Error)]
pub enum StreamingChangesError {
    #[error("Failed to execute the changes on Redpanda")]
    RedpandaChanges(#[from] redpanda::RedpandaChangesError),
}

/// This function validates changes to be made to the streaming infrastructure
/// by delegating validation to the appropriate streaming engine (currently only Redpanda).
///
/// # Arguments
/// * `changes` - A slice of StreamingChange enums representing the changes to validate
///
/// # Returns
/// * `Ok(())` if validation succeeds
/// * `Err(StreamingChangesError)` if validation fails
pub fn validate_changes(changes: &[StreamingChange]) -> Result<(), StreamingChangesError> {
    redpanda::validate_changes(changes)?;

    Ok(())
}

/// This method dispatches the execution of the changes to the right streaming engine.
/// When we have multiple streams (Redpanda, RabbitMQ ...) this is where it goes.
pub async fn execute_changes(
    project: &Project,
    changes: &[StreamingChange],
) -> Result<(), StreamingChangesError> {
    redpanda::execute_changes(project, changes).await?;

    Ok(())
}
