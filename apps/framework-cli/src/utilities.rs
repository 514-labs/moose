use std::path::Path;

pub mod auth;
pub mod capture;
pub mod constants;
pub mod decode_object;
pub mod docker;
pub mod git;
pub mod package_managers;
pub mod retry;
pub mod system;
pub mod validate_passthrough;

pub trait PathExt {
    fn ext_is_supported_lang(&self) -> bool;
    fn ext_is_script_config(&self) -> bool;
}
impl PathExt for Path {
    fn ext_is_supported_lang(&self) -> bool {
        self.extension().is_some_and(|ext| {
            ext == constants::TYPESCRIPT_FILE_EXTENSION || ext == constants::PYTHON_FILE_EXTENSION
        })
    }

    fn ext_is_script_config(&self) -> bool {
        self.to_str().is_some_and(|path| {
            path.contains(constants::SCRIPTS_DIR)
                && self
                    .file_name()
                    .is_some_and(|name| name == constants::CLI_CONFIG_FILE)
        })
    }
}

pub const fn _true() -> bool {
    true
}
