use std::{
    fs::File,
    io::Write,
    path::{self, PathBuf},
};
mod mapper;
mod templates;

use crate::project::Project;

use self::templates::{PackageJsonTemplate, TsConfigTemplate};

use super::{
    directories::get_igloo_directory,
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

    pub fn from_project(project: &Project) -> Self {
        Self {
            name: format!("{}-sdk", project.name.clone()),
        }
    }
}

fn write_config_to_file(path: PathBuf, code: String) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;
    file.write_all(code.as_bytes())
}

pub fn generate_ts_sdk(
    project: &Project,
    ts_objects: Vec<TypescriptObjects>,
) -> Result<(), std::io::Error> {
    let igloo_dir = get_igloo_directory(project.clone())?;

    let package = TypescriptPackage::from_project(project);
    let package_json_code = PackageJsonTemplate::new(&package);
    let ts_config_code = TsConfigTemplate::new();
    let index_code = IndexTemplate::new(&ts_objects);

    // This needs to write to the root of the NPM folder... creating in the current project location for now
    let sdk_dir = igloo_dir.join(package.name);
    std::fs::create_dir_all(sdk_dir.clone())?;

    write_config_to_file(sdk_dir.join("package.json"), package_json_code)?;
    write_config_to_file(sdk_dir.join("tsconfig.json"), ts_config_code)?;

    languages::write_code_to_file(
        languages::SupportedLanguages::Typescript,
        sdk_dir.join("index.ts"),
        index_code,
    )?;

    for obj in ts_objects {
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
    Ok(())
}
