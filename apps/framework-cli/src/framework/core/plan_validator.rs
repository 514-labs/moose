use crate::infrastructure::stream;

use super::plan::InfraPlan;

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Some of the changes derived for the streaming engine are invalid")]
    StreamingChange(#[from] stream::StreamingChangesError),
}

pub fn validate(plan: &InfraPlan) -> Result<(), ValidationError> {
    stream::validate_changes(&plan.changes.streaming_engine_changes)?;

    Ok(())
}
