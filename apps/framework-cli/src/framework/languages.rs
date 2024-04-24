use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::project::Project;

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

pub fn create_models_dir(project: &Project) -> Result<PathBuf, std::io::Error> {
    let internal_dir = project.internal_dir()?;
    std::fs::create_dir_all(internal_dir.join("models").clone())?;
    Ok(internal_dir)
}

pub fn get_models_dir(project: &Project) -> Result<PathBuf, std::io::Error> {
    let internal_dir = project.internal_dir()?;
    let models_dir = internal_dir.join("models");

    if models_dir.exists() {
        Ok(models_dir)
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            "Models directory not found",
        ))
    }
}
