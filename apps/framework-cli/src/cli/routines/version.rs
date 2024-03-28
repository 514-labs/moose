use regex::Regex;
use std::fs;
use std::sync::Arc;

use toml_edit::{table, value, DocumentMut, Item, Value};

use crate::cli::display::Message;
use crate::cli::routines::{Routine, RoutineFailure, RoutineSuccess};
use crate::project::{Project, PROJECT};
use crate::utilities::constants::{PACKAGE_JSON, PROJECT_CONFIG_FILE};
use crate::utilities::git::current_commit_hash;

pub struct BumpVersion {
    project: Arc<Project>,
    new_version: String,
}

impl BumpVersion {
    pub fn new(project: Arc<Project>, new_version: String) -> Self {
        Self {
            project,
            new_version,
        }
    }
}
impl Routine for BumpVersion {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        add_current_version_to_config(self.project.version()).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!("to update {}", PROJECT_CONFIG_FILE),
                ),
                err,
            )
        })?;

        bump_package_json_version(self.project.version(), &self.new_version).map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), format!("to update {}", PACKAGE_JSON)),
                err,
            )
        })?;

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            "bumped version".to_string(),
        )))
    }
}

fn bump_package_json_version(current_version: &str, new_version: &str) -> anyhow::Result<()> {
    let contents = fs::read_to_string(PACKAGE_JSON)?;

    // rather than parsing it, this keeps the formatting and field ordering of package.json
    let updated = Regex::new(&format!(
        "(\"version\":[ \\n\\r\\t]+)\"{}\"",
        current_version
    ))?
    .replace(&contents, format!("$1\"{}\"", new_version));

    fs::write(PACKAGE_JSON, updated.to_string())?;

    Ok(())
}

fn add_current_version_to_config(current_version: &str) -> anyhow::Result<()> {
    let commit = current_commit_hash(&PROJECT.lock().unwrap())?;

    let contents = fs::read_to_string(PROJECT_CONFIG_FILE)?;
    let mut doc = contents.parse::<DocumentMut>()?;

    match doc.get_mut("supported_old_versions") {
        Some(Item::Table(table)) => {
            table.insert(current_version, value(commit));
        }
        Some(Item::Value(Value::InlineTable(table))) => {
            table.insert(current_version, commit.into());
        }
        _ => {
            doc["supported_old_versions"] = table();
            doc["supported_old_versions"][current_version] = value(commit);
        }
    }

    fs::write(PROJECT_CONFIG_FILE, doc.to_string())?;
    Ok(())
}
