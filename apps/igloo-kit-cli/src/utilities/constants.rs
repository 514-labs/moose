pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const PROJECT_CONFIG_FILE: &str = "project.toml";

pub const CLI_CONFIG_FILE: &str = ".igloo-config.toml";
pub const CLI_USER_DIRECTORY: &str = ".igloo";
pub const CLI_PROJECT_INTERNAL_DIR: &str = ".igloo";

pub const SCHEMAS_DIR: &str = "datamodels";

pub const PANDA_NETWORK: &str = "panda-house";

pub const CLICKHOUSE_CONTAINER_NAME: &str = "clickhousedb-1";
pub const CONSOLE_CONTAINER_NAME: &str = "console-1";
pub const REDPANDA_CONTAINER_NAME: &str = "redpanda-1";

pub const APP_DIR: &str = "app";
pub const APP_DIR_LAYOUT: [&str; 7] = [
    "ingestion_points",
    SCHEMAS_DIR,
    "flows",
    "insights",
    "insights/dashboards",
    "insights/models",
    "insights/metrics",
];
