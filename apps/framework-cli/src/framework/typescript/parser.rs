use std::fs;
use std::io::ErrorKind::NotFound;
use std::path::Path;
use std::process::Command;

use serde_json::{json, Value};

use crate::framework::data_model::parser::FileObjects;
use crate::project::Project;

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse the typescript file")]
#[non_exhaustive]
pub enum TypescriptParsingError {
    #[error("Failure setting up the file structure")]
    FileSystemError(#[from] std::io::Error),
    TypescriptCompilerError(Option<std::io::Error>),
    #[error("Typescript Parser - Unsupported data type: {type_name}")]
    UnsupportedDataTypeError {
        type_name: String,
    },

    #[error("Invalid output from compiler plugin. Possible incompatible versions between moose-lib and moose-cli")]
    DeserializationError(#[from] serde_json::Error),

    OtherError {
        message: String,
    },
}

pub fn extract_data_model_from_file(
    path: &Path,
    project: &Project,
    version: &str,
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
        .current_dir(&project.project_location)
        .spawn()?
        .wait()
        .map_err(|err| TypescriptParsingError::TypescriptCompilerError(Some(err)))?;
    if !ts_return_code.success() {
        return Err(TypescriptParsingError::TypescriptCompilerError(None));
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
    .map_err(|e| TypescriptParsingError::OtherError {
        message: format!("Unable to read output of compiler: {}", e),
    })?;

    let mut output_json = serde_json::from_slice::<Value>(&output)
        .map_err(|_| TypescriptParsingError::TypescriptCompilerError(None))?;
    if let Some(error_type) = output_json.get("error_type") {
        if let Some(error_type) = error_type.as_str() {
            if error_type == "unknown_type" {
                let type_name = output_json
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                return Err(TypescriptParsingError::UnsupportedDataTypeError { type_name });
            } else if error_type == "unsupported_enum" {
                return Err(TypescriptParsingError::OtherError {
                    message: "We do not allow to mix String enums with Number based enums, please choose one".to_string()
                });
            }
        }
    }

    // There is probably a better way to do this by communicating to the underlying
    // process. But for now, we will just add the version and file path to the output
    if let Some(data_models) = output_json.get_mut("models") {
        if let Some(data_models) = data_models.as_array_mut() {
            for data_model in data_models {
                if let Some(dm) = data_model.as_object_mut() {
                    dm.insert("version".to_string(), version.into());
                    dm.insert(
                        "file_path".to_string(),
                        path.to_string_lossy().to_string().into(),
                    );
                }
            }
        }
    }

    Ok(serde_json::from_value(output_json)?)
}

#[cfg(test)]
mod tests {
    use crate::framework::languages::SupportedLanguages;
    use crate::framework::{
        data_model::parser::parse_data_model_file, typescript::parser::extract_data_model_from_file,
    };
    use crate::project::Project;
    use lazy_static::lazy_static;
    use std::path::PathBuf;
    use std::process::Command;

    fn pnpm_moose_lib(cmd_action: fn(&mut Command) -> &mut Command) {
        let mut cmd = Command::new("pnpm");
        cmd_action(&mut cmd)
            .arg("--filter=moose-lib")
            .current_dir("../../")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    lazy_static! {
        static ref TEST_PROJECT: Project = {
            pnpm_moose_lib(|cmd| cmd.arg("i").arg("--frozen-lockfile"));

            pnpm_moose_lib(|cmd| cmd.arg("run").arg("build"));

            Command::new("npm")
                .arg("i")
                .current_dir("./tests/test_project")
                .spawn()
                .unwrap()
                .wait()
                .unwrap();

            Command::new("rm")
                .arg("-rf")
                .arg("./tests/test_project/node_modules/@514labs/moose-lib/dist/")
                .spawn()
                .unwrap()
                .wait()
                .unwrap();

            Command::new("cp")
                .arg("-r")
                .arg("../../packages/ts-moose-lib/dist/")
                .arg("./tests/test_project/node_modules/@514labs/moose-lib/dist/")
                .spawn()
                .unwrap()
                .wait()
                .unwrap();

            Project::new(
                &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test_project"),
                "testing".to_string(),
                SupportedLanguages::Typescript,
            )
        };
    }

    #[test]
    fn test_parse_schema_file() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/psl/simple.prisma");

        let result = parse_data_model_file(&test_file, "", &TEST_PROJECT);
        assert!(result.is_ok());
    }

    #[test]
    #[serial_test::serial(tspc)]
    fn test_ts_mapper() {
        let test_file = TEST_PROJECT.data_models_dir().join("simple.ts");

        let result = extract_data_model_from_file(&test_file, &TEST_PROJECT, "");

        assert!(result.is_ok());
        println!("{:?}", result.unwrap().models)
    }

    #[test]
    #[serial_test::serial(tspc)]
    fn test_parse_typescript_file() {
        let test_file = TEST_PROJECT.data_models_dir().join("simple.ts");

        let result = extract_data_model_from_file(&test_file, &TEST_PROJECT, "");

        assert!(result.is_ok());
    }

    #[test]
    #[serial_test::serial(tspc)]
    fn test_parse_import_typescript_file() {
        let test_file = TEST_PROJECT.data_models_dir().join("import.ts");

        let result = extract_data_model_from_file(&test_file, &TEST_PROJECT, "");
        assert!(result.is_ok());
    }

    #[test]
    #[serial_test::serial(tspc)]
    fn test_parse_extend_typescript_file() {
        let test_file = TEST_PROJECT.data_models_dir().join("extend.m.ts");

        let result = extract_data_model_from_file(&test_file, &TEST_PROJECT, "");
        assert!(result.is_ok());
    }

    #[test]
    #[serial_test::serial(tspc)]
    fn test_ts_syntax_error() {
        let test_file = TEST_PROJECT.data_models_dir().join("syntax_error.ts");

        // The TS compiler prints this, which is forwarded to the user's console
        // app/datamodels/syntax_error.ts(7,23): error TS1005: ',' expected.
        let result = extract_data_model_from_file(&test_file, &TEST_PROJECT, "");
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Failed to parse the typescript file"
        );
    }

    #[test]
    #[serial_test::serial(tspc)]
    fn test_ts_missing_type() {
        let test_file = TEST_PROJECT.data_models_dir().join("type_missing.ts");

        let result = extract_data_model_from_file(&test_file, &TEST_PROJECT, "");
        assert!(result.is_err());
        // The TS compiler prints this, which is forwarded to the user's console
        // app/datamodels/type_missing.ts(2,5): error TS7008: Member 'foo' implicitly has an 'any' type.
        assert_eq!(
            result.err().unwrap().to_string(),
            "Failed to parse the typescript file"
        );
    }
}
