use crate::cli::display::{Message, MessageType};
use crate::cli::routines::RoutineFailure;
use crate::framework::languages::SupportedLanguages;
use crate::framework::python::generate::tables_to_python;
use crate::framework::typescript::generate::tables_to_typescript;
use crate::infrastructure::olap::clickhouse::ConfiguredDBClient;
use crate::infrastructure::olap::OlapOperations;
use crate::utilities::constants::{APP_DIR, PYTHON_MAIN_FILE, TYPESCRIPT_MAIN_FILE};
use log::debug;
use reqwest::Url;
use std::borrow::Cow;
use std::env;
use std::io::Write;
use std::path::Path;

pub async fn db_to_dmv2(remote_url: &str, dir_path: &Path) -> Result<(), RoutineFailure> {
    let mut url = Url::parse(remote_url).map_err(|e| {
        RoutineFailure::error(Message::new(
            "Invalid URL".to_string(),
            format!("Failed to parse remote_url '{remote_url}': {e}"),
        ))
    })?;

    if url.scheme() == "clickhouse" {
        debug!("Only HTTP(s) supported. Transforming native protocol connection string.");
        let is_secure = match (url.host_str(), url.port()) {
            (_, Some(9000)) => false,
            (_, Some(9440)) => true,
            (Some(host), _) if host == "localhost" || host == "127.0.0.1" => false,
            _ => true,
        };
        let (new_port, new_scheme) = if is_secure {
            (8443, "https")
        } else {
            (8123, "http")
        };
        // cannot set_scheme from clickhouse to http(s)
        url = Url::parse(&remote_url.replacen("clickhouse", new_scheme, 1)).unwrap();
        url.set_port(Some(new_port)).unwrap();

        let path_segments = url.path().split('/').collect::<Vec<&str>>();
        if path_segments.len() == 2 && path_segments[0].is_empty() {
            let database = path_segments[1].to_string(); // to_string to end the borrow
            url.set_path("");
            url.query_pairs_mut().append_pair("database", &database);
        };

        let display_url = if url.password().is_some() {
            let mut cloned = url.clone();
            cloned.set_password(Some("******")).unwrap();
            Cow::Owned(cloned)
        } else {
            Cow::Borrowed(&url)
        };
        show_message!(
            MessageType::Highlight,
            Message {
                action: "Protocol".to_string(),
                details: format!("native protocol detected. Converting to HTTP(s): {display_url}"),
            }
        );
    }

    let mut client = clickhouse::Client::default().with_url(remote_url);
    let url_username = url.username();
    let url_username = if !url_username.is_empty() {
        url_username.to_string()
    } else {
        match url.query_pairs().find(|(key, _)| key == "user") {
            None => String::new(),
            Some((_, v)) => v.to_string(),
        }
    };
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
                .open(format!("{APP_DIR}/{TYPESCRIPT_MAIN_FILE}"))
                .map_err(|e| {
                    RoutineFailure::new(
                        Message::new(
                            "Failure".to_string(),
                            format!("opening {TYPESCRIPT_MAIN_FILE}"),
                        ),
                        e,
                    )
                })?;

            writeln!(file, "\n\n{table_definitions}")
        }
        SupportedLanguages::Python => {
            let table_definitions = tables_to_python(&tables);
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(format!("{APP_DIR}/{PYTHON_MAIN_FILE}"))
                .map_err(|e| {
                    RoutineFailure::new(
                        Message::new("Failure".to_string(), format!("opening {PYTHON_MAIN_FILE}")),
                        e,
                    )
                })?;

            writeln!(file, "\n\n{table_definitions}")
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
