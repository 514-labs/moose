pub const PROJECT_CONFIG_FILE: &str = "project.toml";

pub const CLI_CONFIG_FILE: &str = ".igloo-config.toml";
pub const CLI_USER_DIRECTORY: &str = ".igloo";
pub const CLI_PROJECT_INTERNAL_DIR: &str = ".igloo";

pub const SCHEMAS_DIR: &str = "datamodels";

pub const PANDA_NETWORK: &str = "panda-house";

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
