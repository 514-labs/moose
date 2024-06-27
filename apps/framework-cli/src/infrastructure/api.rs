use tokio::sync::mpsc::Sender;

use crate::framework::core::infrastructure_map::ApiChange;

#[derive(Debug, thiserror::Error)]
pub enum ApiChangeError {
    #[error("Could not send the error to the api to be executed")]
    Send(#[from] tokio::sync::mpsc::error::SendError<ApiChange>),
}

pub async fn execute_changes(
    api_changes: &[ApiChange],
    api_changes_channel: Sender<ApiChange>,
) -> Result<(), ApiChangeError> {
    for api_change in api_changes.iter() {
        api_changes_channel.send(api_change.clone()).await?;
    }

    Ok(())
}
