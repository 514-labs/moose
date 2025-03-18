use crate::{infrastructure::stream, project::Project};

use super::plan::InfraPlan;

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Some of the changes derived for the streaming engine are invalid")]
    StreamingChange(#[from] stream::StreamingChangesError),
}

pub fn validate(project: &Project, plan: &InfraPlan) -> Result<(), ValidationError> {
    stream::validate_changes(project, &plan.changes.streaming_engine_changes)?;

    Ok(())
}
