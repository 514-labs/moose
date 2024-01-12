use std::fs::File;
use std::io::{prelude::*, Error, ErrorKind};
use std::path::PathBuf;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::project::Project;

use super::schema::UnsupportedDataTypeError;

#[derive(ValueEnum, Copy, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum SupportedLanguages {
    #[value(name = "ts")]
    Typescript,
}

impl std::fmt::Display for SupportedLanguages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SupportedLanguages::Typescript => "ts",
        };

        s.fmt(f)
    }
}

impl std::str::FromStr for SupportedLanguages {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ts" => Ok(SupportedLanguages::Typescript),
            _ => Err(format!("{} is not a supported language", s)),
        }
    }
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
    }
}

pub fn create_models_dir(project: Project) -> Result<PathBuf, std::io::Error> {
    let igloo_dir = project.internal_dir()?;
    std::fs::create_dir_all(igloo_dir.join("models").clone())?;
    Ok(igloo_dir)
}

pub fn get_models_dir(project: Project) -> Result<PathBuf, std::io::Error> {
    let igloo_dir = project.internal_dir()?;
    let models_dir = igloo_dir.join("models");

    if models_dir.exists() {
        Ok(models_dir)
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            "Models directory not found",
        ))
    }
}
