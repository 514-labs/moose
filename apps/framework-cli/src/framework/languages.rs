use clap::ValueEnum;
use serde::{Deserialize, Serialize};

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
