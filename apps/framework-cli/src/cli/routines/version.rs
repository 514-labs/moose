use regex::Regex;
use std::fs;

use toml_edit::{table, value, DocumentMut, Item, Value};

use crate::cli::display::Message;
use crate::cli::routines::{RoutineFailure, RoutineSuccess};
use crate::framework::languages::SupportedLanguages;
use crate::project::Project;
use crate::utilities::constants::{
    OLD_PROJECT_CONFIG_FILE, PACKAGE_JSON, PROJECT_CONFIG_FILE, SETUP_PY,
};
use crate::utilities::git::current_commit_hash;

pub fn bump_version(
    project: &Project,
    new_version: String,
) -> Result<RoutineSuccess, RoutineFailure> {
    add_current_version_to_config(project, project.cur_version()).map_err(|err| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                format!(
                    "to update {} or {}",
                    OLD_PROJECT_CONFIG_FILE, PROJECT_CONFIG_FILE
                ),
            ),
            err,
        )
    })?;

    match project.language {
        SupportedLanguages::Python => {
            bump_setup_py_version(project.cur_version(), &new_version).map_err(|err| {
                RoutineFailure::new(
                    Message::new("Failed".to_string(), format!("to update {}", SETUP_PY)),
                    err,
                )
            })?;
        }
        SupportedLanguages::Typescript => {
            bump_package_json_version(project.cur_version(), &new_version).map_err(|err| {
                RoutineFailure::new(
                    Message::new("Failed".to_string(), format!("to update {}", PACKAGE_JSON)),
                    err,
                )
            })?;
        }
    }

    Ok(RoutineSuccess::success(Message::new(
        "Bumped".to_string(),
        "version".to_string(),
    )))
}

fn bump_setup_py_version(current_version: &str, new_version: &str) -> anyhow::Result<()> {
    let contents = fs::read_to_string(SETUP_PY)?;

    // rather than parsing it, this keeps the formatting and field ordering of setup.py
    let updated = Regex::new(&format!("(version='{}')", current_version))?
        .replace(&contents, format!("version='{}'", new_version));

    fs::write(SETUP_PY, updated.to_string())?;

    Ok(())
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

    let contents = if project.project_location.join(PROJECT_CONFIG_FILE).exists() {
        fs::read_to_string(PROJECT_CONFIG_FILE)?
    } else {
        fs::read_to_string(OLD_PROJECT_CONFIG_FILE)?
    };

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

    if project.project_location.join(PROJECT_CONFIG_FILE).exists() {
        fs::write(PROJECT_CONFIG_FILE, doc.to_string())?;
    } else {
        fs::write(OLD_PROJECT_CONFIG_FILE, doc.to_string())?;
    }

    Ok(())
}
