//! # Logger Module
//!
//! This module provides logging functionality for the application.
//!
//! ## Components
//!
//! - `LoggerLevel`: An enumeration representing the different levels of logging: DEBUG, INFO, WARN, and ERROR.
//! - `LoggerSettings`: A struct that holds the settings for the logger, including the log file's name and the logging level.
//! - `setup_logging`: A function used to set up the logging system with the provided settings.
//!
//! ## Usage
//!
//! The logger is configured by creating a `LoggerSettings` instance and passing it to the `setup_logging` function.
//! The `LoggerSettings` can be configured with a log file and a log level. If these are not provided, default values are used.
//! The default log file is "cli.log" in the user's directory, and the default log level is INFO.
//! Use the macros to write to the log file.
//!
//! The log levels have the following uses:
//! - `DEBUG`: Use this level for detailed information typically of use only when diagnosing problems. You would usually only expect to see these logs in a development environment. For example, you might log method entry/exit points, variable values, query results, etc.
//! - `INFO`: Use this level to confirm that things are working as expected. This is the default log level and will give you general operational insights into the application behavior. For example, you might log start/stop of a process, configuration details, successful completion of significant transactions, etc.
//! - `WARN`: Use this level when something unexpected happened in the system, or there might be a problem in the near future (like 'disk space low'). The software is still working as expected, so it's not an error. For example, you might log deprecated API usage, poor performance issues, retrying an operation, etc.
//! - `ERROR`: Use this level when the system is in distress, customers are probably being affected but the program is not terminated. An operator should definitely look into it. For example, you might log exceptions, potential data inconsistency, or system overloads.
//!
//! ## Example
//!
//! ```rust
//! debug!("This is a DEBUG message. Typically used for detailed information useful in a development environment.");
//! info!("This is an INFO message. Used to confirm that things are working as expected.");
//! warn!("This is a WARN message. Indicates something unexpected happened or there might be a problem in the near future.");
//! error!("This is an ERROR message. Used when the system is in distress, customers are probably being affected but the program is not terminated.");
//! ```
//!
//! ## Future Enhancements
//!
//! - Log file rotation: The log file should be rotated after it reaches a certain size to manage disk space.

use log::{LevelFilter, Metadata, Record};
use std::time::{Duration, SystemTime};

use opentelemetry::logs::Logger;
use opentelemetry::KeyValue;
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_http::hyper::HyperClient;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::logs::Config;
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use serde::Deserialize;

use crate::utilities::constants::{CONTEXT, CTX_SESSION_ID};

use super::settings::user_directory;

const LOG_FILE: &str = "cli.log";

#[derive(Deserialize, Debug, Clone)]
pub enum LoggerLevel {
    #[serde(alias = "DEBUG", alias = "debug")]
    Debug,
    #[serde(alias = "INFO", alias = "info")]
    Info,
    #[serde(alias = "WARN", alias = "warn")]
    Warn,
    #[serde(alias = "ERROR", alias = "error")]
    Error,
}

impl LoggerLevel {
    pub fn to_log_level(&self) -> log::LevelFilter {
        match self {
            LoggerLevel::Debug => log::LevelFilter::Debug,
            LoggerLevel::Info => log::LevelFilter::Info,
            LoggerLevel::Warn => log::LevelFilter::Warn,
            LoggerLevel::Error => log::LevelFilter::Error,
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum LogFormat {
    Json,
    Text,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LoggerSettings {
    #[serde(default = "default_log_file")]
    pub log_file: String,
    #[serde(default = "default_log_level")]
    pub level: LoggerLevel,
    #[serde(default = "default_log_stdout")]
    pub stdout: bool,

    #[serde(default = "default_log_format")]
    pub format: LogFormat,

    pub export_to: Option<reqwest::Url>,
}

fn default_log_file() -> String {
    let mut dir = user_directory();
    dir.push(LOG_FILE);
    dir.to_str().unwrap().to_string()
}

fn default_log_level() -> LoggerLevel {
    LoggerLevel::Info
}

fn default_log_stdout() -> bool {
    false
}

fn default_log_format() -> LogFormat {
    LogFormat::Text
}

impl Default for LoggerSettings {
    fn default() -> Self {
        LoggerSettings {
            log_file: default_log_file(),
            level: default_log_level(),
            stdout: default_log_stdout(),
            format: default_log_format(),
            export_to: None,
        }
    }
}

// TODO ensure that the log file rotates after a certain size
pub fn setup_logging(settings: &LoggerSettings, machine_id: &str) -> Result<(), fern::InitError> {
    let session_id = CONTEXT.get(CTX_SESSION_ID).unwrap();

    let base_config = fern::Dispatch::new().level(settings.level.to_log_level());

    let format_config = if settings.format == LogFormat::Text {
        fern::Dispatch::new().format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {} {} - {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                &session_id,
                record.target(),
                message
            ))
        })
    } else {
        fern::Dispatch::new().format(move |out, message, record| {
            out.finish(format_args!(
                "{}",
                serde_json::to_string(&serde_json::json!(
                    {
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "severity": record.level(),
                        "session_id": &session_id,
                        "target": record.target(),
                        "message": message,
                    }
                ))
                .expect("formatting `serde_json::Value` with string keys never fails")
            ))
        })
    };

    let output_config = if settings.stdout {
        format_config.chain(std::io::stdout())
    } else {
        format_config.chain(fern::log_file(&settings.log_file)?)
    };

    let output_config = match &settings.export_to {
        None => output_config,
        Some(otel_endpoint) => {
            let https = hyper_tls_0_5::HttpsConnector::new();
            let client = hyper_0_14::Client::builder().build::<_, hyper_0_14::Body>(https);

            let otel_exporter = opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(otel_endpoint.clone())
                .with_protocol(Protocol::HttpJson)
                .with_http_client(HyperClient::new_with_timeout(
                    client,
                    Duration::from_millis(5000),
                ))
                .with_timeout(Duration::from_millis(5000))
                .build_log_exporter()
                .unwrap();

            let logger_provider = LoggerProvider::builder()
                .with_config(Config::default().with_resource(Resource::new(vec![
                    KeyValue::new(SERVICE_NAME, "moose-cli"),
                    KeyValue::new("session_id", session_id.as_str()),
                    KeyValue::new("machine_id", String::from(machine_id)),
                ])))
                .with_batch_exporter(otel_exporter, opentelemetry_sdk::runtime::Tokio)
                .build();

            let logger: Box<dyn log::Log> = Box::new(TargetToKvLogger {
                inner: OpenTelemetryLogBridge::new(&logger_provider),
            });

            fern::Dispatch::new().chain(output_config).chain(
                fern::Dispatch::new()
                    // to prevent exporter recursively calls logging and thus itself
                    .level(LevelFilter::Off)
                    .level_for("moose_cli", settings.level.to_log_level())
                    .chain(logger),
            )
        }
    };
    base_config.chain(output_config).apply()?;

    Ok(())
}

struct TargetToKvLogger<P, L>
where
    P: opentelemetry::logs::LoggerProvider<Logger = L> + Send + Sync,
    L: Logger + Send + Sync,
{
    inner: OpenTelemetryLogBridge<P, L>,
}

impl<P, L> log::Log for TargetToKvLogger<P, L>
where
    P: opentelemetry::logs::LoggerProvider<Logger = L> + Send + Sync,
    L: Logger + Send + Sync,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        let mut with_target = record.to_builder();
        let kvs: &dyn log::kv::Source = &("target", record.target());
        with_target.key_values(kvs);
        self.inner.log(&with_target.build());
    }

    fn flush(&self) {
        self.inner.flush()
    }
}
