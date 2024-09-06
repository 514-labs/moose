pub mod aggregation;
pub mod consumption;
pub mod executor;
pub mod parser;
pub mod streaming;
pub mod templates;
pub mod utils;

pub fn version_to_identifier(version: &str) -> String {
    format!("v{}", version.replace('.', "_"))
}
