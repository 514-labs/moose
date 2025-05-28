use crate::cli::display::{Message, MessageType};
use crate::cli::routines::RoutineFailure;
use crate::framework::languages::SupportedLanguages;
use crate::framework::python::generate::tables_to_python;
use crate::framework::typescript::generate::tables_to_typescript;
use crate::infrastructure::olap::clickhouse::ConfiguredDBClient;
use crate::infrastructure::olap::OlapOperations;
use crate::utilities::clickhouse_url::normalize_clickhouse_url;
use crate::utilities::constants::{APP_DIR, PYTHON_MAIN_FILE, TYPESCRIPT_MAIN_FILE};
use std::env;
use std::io::Write;
use std::path::Path;

pub async fn db_to_dmv2(remote_url: &str, dir_path: &Path) -> Result<(), RoutineFailure> {
    let url = normalize_clickhouse_url(remote_url).map_err(|e| {
        RoutineFailure::error(Message::new(
            "Invalid URL".to_string(),
            format!("Failed to parse remote_url '{}': {}", remote_url, e),
        ))
    })?;

    let mut client = clickhouse::Client::default().with_url(remote_url);
    let url_username = url.username();
    if !url_username.is_empty() {
        client = client.with_user(url_username)
    }
    if let Some(password) = url.password() {
        client = client.with_password(password);
    }

    let url_db = url
        .query_pairs()
        .filter_map(|(k, v)| {
            if k == "database" {
                Some(v.to_string())
            } else {
                None
            }
        })
        .last();

    let client = ConfiguredDBClient {
        client,
        config: Default::default(),
    };

    let db = match url_db {
        None => client
            .client
            .query("select database()")
            .fetch_one::<String>()
            .await
            .map_err(|e| {
                RoutineFailure::new(
                    Message::new("Failure".to_string(), "fetching database".to_string()),
                    e,
                )
            })?,
        Some(db) => db,
    };
    env::set_current_dir(dir_path).map_err(|e| {
        RoutineFailure::new(
            Message::new("Failure".to_string(), "changing directory".to_string()),
            e,
        )
    })?;

    let project = crate::cli::load_project()?;
    let (tables, unsupported) = client.list_tables(&db, &project).await.map_err(|e| {
        RoutineFailure::new(
            Message::new("Failure".to_string(), "listing tables".to_string()),
            e,
        )
    })?;

    if !unsupported.is_empty() {
        show_message!(
            MessageType::Highlight,
            Message {
                action: "Table(s)".to_string(),
                details: format!(
                    "with types unsupported: {}",
                    unsupported
                        .iter()
                        .map(|t| t.name.as_str())
                        .collect::<Vec<&str>>()
                        .join(", ")
                ),
            }
        );
    }

    match project.language {
        SupportedLanguages::Typescript => {
            let table_definitions = tables_to_typescript(&tables);
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(format!("{}/{}", APP_DIR, TYPESCRIPT_MAIN_FILE))
                .map_err(|e| {
                    RoutineFailure::new(
                        Message::new(
                            "Failure".to_string(),
                            format!("opening {}", TYPESCRIPT_MAIN_FILE),
                        ),
                        e,
                    )
                })?;

            writeln!(file, "\n\n{}", table_definitions)
        }
        SupportedLanguages::Python => {
            let table_definitions = tables_to_python(&tables);
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(format!("{}/{}", APP_DIR, PYTHON_MAIN_FILE))
                .map_err(|e| {
                    RoutineFailure::new(
                        Message::new(
                            "Failure".to_string(),
                            format!("opening {}", PYTHON_MAIN_FILE),
                        ),
                        e,
                    )
                })?;

            writeln!(file, "\n\n{}", table_definitions)
        }
    }
    .map_err(|e| {
        RoutineFailure::new(
            Message::new(
                "Failure".to_string(),
                "writing table definitions".to_string(),
            ),
            e,
        )
    })?;
    Ok(())
}
