use lazy_static::lazy_static;
use std::collections::HashMap;
use uuid::Uuid;

pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const PACKAGE_JSON: &str = "package.json";
pub const PROJECT_CONFIG_FILE: &str = "project.toml";

pub const CLI_CONFIG_FILE: &str = "config.toml";
pub const CLI_USER_DIRECTORY: &str = ".moose";
pub const CLI_PROJECT_INTERNAL_DIR: &str = ".moose";
pub const CLI_INTERNAL_VERSIONS_DIR: &str = "versions";
pub const CLI_DEV_REDPANDA_VOLUME_DIR: &str = "redpanda";
pub const CLI_DEV_CLICKHOUSE_VOLUME_DIR_LOGS: &str = "clickhouse/logs";
pub const CLI_DEV_CLICKHOUSE_VOLUME_DIR_DATA: &str = "clickhouse/data";
pub const CLI_DEV_CLICKHOUSE_VOLUME_DIR_CONFIG_SCRIPTS: &str = "clickhouse/configs/scripts";
pub const CLI_DEV_CLICKHOUSE_VOLUME_DIR_CONFIG_USERS: &str = "clickhouse/configs/users";

pub const SCHEMAS_DIR: &str = "datamodels";
pub const FLOWS_DIR: &str = "flows";
pub const AGGREGATES_DIR: &str = "aggregates";
pub const SAMPLE_FLOWS_SOURCE: &str = "UserActivity";
pub const SAMPLE_FLOWS_DEST: &str = "ParsedActivity";

pub const CLICKHOUSE_CONTAINER_NAME: &str = "clickhousedb-1";
pub const CONSOLE_CONTAINER_NAME: &str = "console-1";
pub const REDPANDA_CONTAINER_NAME: &str = "redpanda-1";

pub const REDPANDA_HOSTS: [&str; 2] = ["redpanda", "localhost"];

pub const APP_DIR: &str = "app";
pub const APP_DIR_LAYOUT: [&str; 2] = [SCHEMAS_DIR, FLOWS_DIR];

pub const GITIGNORE: &str = ".gitignore";

pub const DENO_DIR: &str = "deno";
pub const DENO_TRANSFORM: &str = "transform.ts";
pub const DENO_AGGREGATE: &str = "aggregates.ts";
pub const FLOW_FILE: &str = "flow.ts";

pub const CTX_SESSION_ID: &str = "session_id";

lazy_static! {
    pub static ref CONTEXT: HashMap<String, String> = {
        let mut map = HashMap::new();
        map.insert(CTX_SESSION_ID.to_string(), Uuid::new_v4().to_string());
        map
    };
}

pub const README_PREFIX: &str = r#"
This is a [MooseJs](https://www.moosejs.com/) project bootstrapped with the 
[`Moose CLI`](https://github.com/514-labs/moose/tree/main/apps/framework-cli).

"#;

pub const TS_INTERFACE_GENERATE_EXT: &str = ".generated.ts";
