use crate::framework::core::check::{CheckerError, SystemChecker};
use async_trait::async_trait;

pub struct TypeScriptChecker;

#[async_trait]
impl SystemChecker for TypeScriptChecker {
    async fn validate(&self) -> Result<(), CheckerError> {
        Ok(())
    }
}
