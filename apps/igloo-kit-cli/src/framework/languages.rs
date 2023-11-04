use std::fs::File;
use std::io::{prelude::*, Error, ErrorKind};
use std::path::PathBuf;

use super::directories::get_igloo_directory;
use super::schema::UnsupportedDataTypeError;

pub enum SupportedLanguages {
    Typescript,
}

pub trait CodeGenerator {
    fn create_code(&self) -> Result<String, UnsupportedDataTypeError>;
}

pub fn write_code_to_file(
    language: SupportedLanguages,
    path: PathBuf,
    code: String,
) -> Result<(), std::io::Error> {
    match language {
        SupportedLanguages::Typescript => {
            let mut file = File::create(path)?;
            file.write_all(code.as_bytes())?;
            Ok(())
        }
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unsupported language",
        )),
    }
}

pub fn create_models_dir() -> Result<PathBuf, std::io::Error> {
    let igloo_dir = get_igloo_directory()?;
    std::fs::create_dir_all(igloo_dir.join("models").clone()).map_err(|err| {
        println!("Failed to create models directory: {}", err);
        err
    })?;
    Ok(igloo_dir)
}

pub fn get_models_dir() -> Result<PathBuf, std::io::Error> {
    let mut igloo_dir = get_igloo_directory()?;
    igloo_dir.push("models");

    if igloo_dir.exists() {
        Ok(igloo_dir)
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            "Models directory not found",
        ))
    }
}
