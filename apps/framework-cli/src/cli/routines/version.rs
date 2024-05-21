use regex::Regex;
use std::fs;

use toml_edit::{table, value, DocumentMut, Item, Value};

use crate::cli::display::Message;
use crate::cli::routines::{RoutineFailure, RoutineSuccess};
use crate::project::Project;
use crate::utilities::constants::{PACKAGE_JSON, PROJECT_CONFIG_FILE};
use crate::utilities::git::current_commit_hash;

pub fn bump_version(
    project: &Project,
    new_version: String,
) -> Result<RoutineSuccess, RoutineFailure> {
    add_current_version_to_config(project, project.version()).map_err(|err| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                format!("to update {}", PROJECT_CONFIG_FILE),
            ),
            err,
        )
    })?;

    bump_package_json_version(project.version(), &new_version).map_err(|err| {
        RoutineFailure::new(
            Message::new("Failed".to_string(), format!("to update {}", PACKAGE_JSON)),
            err,
        )
    })?;

    Ok(RoutineSuccess::success(Message::new(
        "Bumped".to_string(),
        "version".to_string(),
    )))
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

fn add_current_version_to_config(project: &Project, current_version: &str) -> anyhow::Result<()> {
    let commit = current_commit_hash(project)?;

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
