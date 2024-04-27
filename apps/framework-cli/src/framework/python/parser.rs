use std::path::PathBuf;

use anyhow::Error;
use rustpython_parser::{ast, Parse};

pub fn extract_data_model_from_file(path: &PathBuf) -> Result<(), Error> {
    let schema_file = std::fs::read_to_string(path)?;
    let ast = ast::Suite::parse(&schema_file, "<embedded>");

    println!("{:#?}", ast);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_schema_file() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/python/simple.py");

        let result = extract_data_model_from_file(&test_file);

        assert!(result.is_ok());
    }
}
