use std::collections::HashMap;
use std::sync::Arc;
use std::{fs::File, io::Write, path::PathBuf};

mod mapper;
mod templates;

use crate::{
    project::Project,
    utilities::{package_managers, system},
};

use self::templates::{PackageJsonTemplate, TsConfigTemplate};

use super::{
    languages::{self, CodeGenerator},
    typescript::{templates::IndexTemplate, SendFunction, TypescriptInterface},
};

#[derive(Debug, Clone)]
pub struct TypescriptObjects {
    pub interface: TypescriptInterface,
    pub send_function: SendFunction,
}

impl TypescriptObjects {
    pub fn new(interface: TypescriptInterface, send_function: SendFunction) -> Self {
        Self {
            interface,
            send_function,
        }
    }
}

pub struct TypescriptPackage {
    name: String,
    // version: String,
    // description: String,
    // author: String,
}

impl TypescriptPackage {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn from_project(project: Arc<Project>) -> Self {
        Self {
            name: format!("{}-sdk", project.name().clone()),
        }
    }
}

fn write_config_to_file(path: PathBuf, code: String) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;
    file.write_all(code.as_bytes())
}

pub fn generate_ts_sdk(
    project: Arc<Project>,
    ts_objects: &HashMap<String, TypescriptObjects>,
) -> Result<PathBuf, std::io::Error> {
    //! Generates a Typescript SDK for the given project and returns the path where the SDK was generated.
    //!
    //! # Arguments
    //! - `project` - The project to generate the SDK for.
    //! - `ts_objects` - The objects to generate the SDK for.
    //!
    //!
    //! # Returns
    //! - `Result<PathBuf, std::io::Error>` - A result containing the path where the SDK was generated.
    //!
    let internal_dir = project.internal_dir()?;

    let package = TypescriptPackage::from_project(project);
    let package_json_code = PackageJsonTemplate::build(&package);
    let ts_config_code = TsConfigTemplate::build();
    let index_code = IndexTemplate::build(ts_objects);

    // This needs to write to the root of the NPM folder... creating in the current project location for now
    let sdk_dir = internal_dir.join(package.name);

    std::fs::remove_dir_all(sdk_dir.clone()).or_else(|err| match err.kind() {
        std::io::ErrorKind::NotFound => Ok(()),
        _ => Err(err),
    })?;

    std::fs::create_dir_all(sdk_dir.clone())?;

    write_config_to_file(sdk_dir.join("package.json"), package_json_code)?;
    write_config_to_file(sdk_dir.join("tsconfig.json"), ts_config_code)?;

    languages::write_code_to_file(
        languages::SupportedLanguages::Typescript,
        sdk_dir.join("index.ts"),
        index_code,
    )?;

    for obj in ts_objects.values() {
        let interface_code = obj.interface.create_code().map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get typescript interface: {:?}", err),
            )
        })?;
        let send_function_code = obj.send_function.create_code().map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get typescript send function: {:?}", err),
            )
        })?;

        languages::write_code_to_file(
            languages::SupportedLanguages::Typescript,
            sdk_dir.join(obj.interface.file_name_with_extension()),
            interface_code,
        )?;
        languages::write_code_to_file(
            languages::SupportedLanguages::Typescript,
            sdk_dir.join(obj.send_function.file_name_with_extension()),
            send_function_code,
        )?;
    }
    Ok(sdk_dir)
}

pub fn move_to_npm_global_dir(sdk_location: &PathBuf) -> Result<PathBuf, std::io::Error> {
    //! Moves the generated SDK to the NPM global directory.
    //!
    //! *** Note *** This here doesn't work for typescript due to package resolution issues.
    //!
    //! # Arguments
    //! - `sdk_location` - The location of the generated SDK.
    //!
    //! # Returns
    //! - `Result<PathBuf, std::io::Error>` - A result containing the path where the SDK was moved to.
    //!
    let global_node_modules = package_managers::get_or_create_global_folder()?;

    system::copy_directory(sdk_location, &global_node_modules)?;

    Ok(global_node_modules)
}
