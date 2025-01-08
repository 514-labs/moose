use crate::utilities::constants;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, Copy, Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub enum SupportedLanguages {
    #[value(name = "typescript", alias = "ts")]
    Typescript,
    #[value(name = "python", alias = "py")]
    Python,
}

impl SupportedLanguages {
    pub fn extension(self) -> &'static str {
        match self {
            SupportedLanguages::Typescript => constants::TYPESCRIPT_FILE_EXTENSION,
            SupportedLanguages::Python => constants::PYTHON_FILE_EXTENSION,
        }
    }
}

impl std::fmt::Display for SupportedLanguages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SupportedLanguages::Typescript => "ts",
            SupportedLanguages::Python => "python",
        };

        s.fmt(f)
    }
}

impl std::str::FromStr for SupportedLanguages {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ts" => Ok(SupportedLanguages::Typescript),
            "python" => Ok(SupportedLanguages::Python),
            _ => Err(format!("{} is not a supported language", s)),
        }
    }
}
