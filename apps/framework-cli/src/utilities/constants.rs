pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const PROJECT_CONFIG_FILE_TS: &str = "package.json";

pub const CLI_CONFIG_FILE: &str = "config.toml";
pub const CLI_USER_DIRECTORY: &str = ".moose";
pub const CLI_PROJECT_INTERNAL_DIR: &str = ".moose";

pub const SCHEMAS_DIR: &str = "datamodels";

pub const CLICKHOUSE_CONTAINER_NAME: &str = "clickhousedb-1";
pub const CONSOLE_CONTAINER_NAME: &str = "console-1";
pub const REDPANDA_CONTAINER_NAME: &str = "redpanda-1";

pub const REDPANDA_HOSTS: [&str; 2] = ["redpanda", "localhost"];

pub const APP_DIR: &str = "app";
pub const APP_DIR_LAYOUT: [&str; 5] = [
    SCHEMAS_DIR,
    "flows",
    "insights",
    "insights/charts",
    "insights/metrics",
];
