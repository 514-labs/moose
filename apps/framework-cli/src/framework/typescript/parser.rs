use std::fs;
use std::io::ErrorKind::NotFound;
use std::path::Path;
use std::process::Command;

use serde_json::json;

use crate::framework::data_model::parser::FileObjects;
use crate::framework::typescript::parser::TypescriptParsingError::TypescriptCompilerError;
use crate::project::Project;

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse the typescript file")]
#[non_exhaustive]
pub enum TypescriptParsingError {
    #[error("Failure setting up the file structure")]
    FileSystemError(#[from] std::io::Error),
    TypescriptCompilerError(Option<std::io::Error>),
    #[error("Invalid output from compiler plugin. Possible incompatible versions between moose-lib and moose-cli")]
    DeserializationError(#[from] serde_json::Error),
}

pub fn extract_data_model_from_file(
    path: &Path,
    project: &Project,
) -> Result<FileObjects, TypescriptParsingError> {
    let internal = project.internal_dir().unwrap();
    let output_dir = internal.join("serialized_datamodels");

    fs::write(
        internal.join("tsconfig.json"),
        json!({
            "compilerOptions":{
                "outDir": "dist", // relative path, so .moose/dist
                "plugins": [{
                    "transform": "../node_modules/@514labs/moose-lib/dist/toDataModels.js"
                }],
                "strict":true
            },
            "include":[path]
        })
        .to_string(),
    )?;
    fs::remove_dir_all(&output_dir).or_else(
        |e| {
            if e.kind() == NotFound {
                Ok(())
            } else {
                Err(e)
            }
        },
    )?;
    let ts_return_code = Command::new("npx")
        .arg("tspc")
        .arg("--project")
        .arg(".moose/tsconfig.json")
        .spawn()?
        .wait()
        .map_err(|err| TypescriptCompilerError(Some(err)))?;
    if !ts_return_code.success() {
        return Err(TypescriptCompilerError(None));
    }
    let output = fs::read(
        output_dir.join(
            path.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .replace(".ts", ".json"),
        ),
    )
    .map_err(|_| TypescriptCompilerError(None))?;

    Ok(serde_json::from_slice(&output)?)
}

#[cfg(test)]
mod tests {
    use crate::framework::{
        data_model::parser::parse_data_model_file, typescript::parser::extract_data_model_from_file,
    };

    #[test]
    fn test_parse_schema_file() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/psl/simple.prisma");

        let result = parse_data_model_file(&test_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ts_mapper() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/ts/simple.ts");

        let result = extract_data_model_from_file(&test_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_typescript_file() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/ts/simple.ts");

        let result = extract_data_model_from_file(&test_file);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_import_typescript_file() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/ts/import.ts");

        let result = extract_data_model_from_file(&test_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_extend_typescript_file() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/ts/extend.m.ts");

        let result = extract_data_model_from_file(&test_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ts_syntax_error() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/ts/syntax_error.ts");

        let result = extract_data_model_from_file(&test_file);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            r#"Typescript Parser - Invalid typescript file, please refer to the documentation for an example of a valid typescript file
Expected ',', got ';'"#
        );
    }

    #[test]
    fn test_ts_missing_type() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/ts/type_missing.ts");

        let result = extract_data_model_from_file(&test_file);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Typescript Parser - Missing type annotation for foo"
        );
    }
}
