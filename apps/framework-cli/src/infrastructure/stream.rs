use crate::{framework::core::infrastructure_map::StreamingChange, project::Project};

pub mod redpanda;
pub mod rpk;

#[derive(Debug, thiserror::Error)]
pub enum StreamingChangesError {
    #[error("Failed to execute the changes on Redpanda")]
    RedpandaChanges(#[from] redpanda::RedpandaChangesError),
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
