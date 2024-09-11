use std::path::Path;

pub mod capture;
pub mod constants;
pub mod docker;
pub mod git;
pub mod package_managers;
pub mod retry;
pub mod system;
pub mod validate_passthrough;

pub trait PathExt {
    fn ext_is_supported_lang(&self) -> bool;
}
impl PathExt for Path {
    fn ext_is_supported_lang(&self) -> bool {
        self.extension().is_some_and(|ext| {
            ext == constants::TYPESCRIPT_FILE_EXTENSION || ext == constants::PYTHON_FILE_EXTENSION
        })
    }
}
