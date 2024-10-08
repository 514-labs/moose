use crate::framework::core::check::{CheckerError, SystemChecker};
use crate::framework::python::utils::check_python_version;
use async_trait::async_trait;

pub struct PythonChecker {
    pub required_version: String,
}

#[async_trait]
impl SystemChecker for PythonChecker {
    async fn validate(&self) -> Result<(), CheckerError> {
        check_python_version(&self.required_version).await
    }
}
