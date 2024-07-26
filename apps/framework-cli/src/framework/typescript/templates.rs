use std::collections::HashSet;

use handlebars::Handlebars;
use serde_json::{json, Value};

use super::generator::{InterfaceFieldType, TSEnum, TypescriptInterface, TypescriptObjects};

#[derive(Debug, thiserror::Error)]
#[error("Failed to generate Typescript code")]
#[non_exhaustive]
pub enum TypescriptRenderingError {
    HandlebarError(#[from] handlebars::RenderError),
}

pub static TS_BASE_STREAMING_FUNCTION_SAMPLE_TEMPLATE: &str = r#"
// Example streaming function: Converts local timestamps in UserActivity data to UTC.

// Imports: Source (UserActivity) and Destination (ParsedActivity) data models.
import { ParsedActivity, UserActivity } from "../datamodels/models";

// The 'run' function transforms UserActivity data to ParsedActivity format.
// For more details on how Moose streaming functions work, see: https://docs.moosejs.com
export default function run(source: UserActivity): ParsedActivity {
  // Convert local timestamp to UTC and return new ParsedActivity object.
  return {
    eventId: source.eventId,  // Retain original event ID.
    userId: "puid" + source.userId,  // Example: Prefix user ID.
    activity: source.activity,  // Copy activity unchanged.
    timestamp: new Date(source.timestamp)  // Convert timestamp to UTC.
  };
}

"#;

pub static BASE_STREAMING_FUNCTION_TEMPLATE: &str = r#"
// Add your models & start the development server to import these types
{{source_import}}
{{destination_import}}

// The 'run' function transforms {{source}} data to {{destination}} format.
// For more details on how Moose streaming functions work, see: https://docs.moosejs.com
export default function run(source: {{source}}): {{destination}} | null {
  return {{destination_object}};
}

"#;

pub static TS_BASE_AGGREGATION_SAMPLE_TEMPLATE: &str = r#"
// Here is a sample aggregation query that calculates the number of daily active users
// based on the number of unique users who complete a sign-in activity each day.

import { Aggregation } from "@514labs/moose-lib";

export default {
  select: ` 
    SELECT 
        uniqState(userId) as dailyActiveUsers,
        toStartOfDay(timestamp) as date
    FROM ParsedActivity_0_0
    WHERE activity = 'Login' 
    GROUP BY toStartOfDay(timestamp)
    `,
  orderBy: "date",
} satisfies Aggregation as Aggregation;

"#;

pub static TS_BASE_BLOCKS_SAMPLE_TEMPLATE: &str = r#"
// Here is a sample aggregation query that calculates the number of daily active users
// based on the number of unique users who complete a sign-in activity each day.

import {
  createAggregation,
  dropAggregation,
  Blocks,
  ClickHouseEngines,
} from "@514labs/moose-lib";

const DESTINATION_TABLE = "DailyActiveUsers";
const MATERIALIZED_VIEW = "DailyActiveUsers_mv";
const SELECT_QUERY = `
SELECT 
    toStartOfDay(timestamp) as date,
    uniqState(userId) as dailyActiveUsers
FROM ParsedActivity_0_0 
WHERE activity = 'Login' 
GROUP BY toStartOfDay(timestamp) 
`;

export default {
  teardown: [
    ...dropAggregation({
      viewName: MATERIALIZED_VIEW,
      tableName: DESTINATION_TABLE,
    }),
  ],
  setup: [
    ...createAggregation({
      tableCreateOptions: {
        name: DESTINATION_TABLE,
        columns: {
          date: "Date",
          dailyActiveUsers: "AggregateFunction(uniq, String)",
        },
        engine: ClickHouseEngines.AggregatingMergeTree,
        orderBy: "date",
      },
      materializedViewName: MATERIALIZED_VIEW,
      select: SELECT_QUERY,
    }),
  ],
} as Blocks;
"#;

pub static BASE_APIS_SAMPLE_TEMPLATE: &str = r#"
// Here is a sample api configuration that creates an API which serves the daily active users materialized view
import { ConsumptionUtil } from "@514labs/moose-lib";

interface QueryParams {
  limit: string;
  minDailyActiveUsers: string;
}

export default async function handle(
  { limit = "10", minDailyActiveUsers = "0" }: QueryParams,
  { client, sql }: ConsumptionUtil
) {
  return client.query(
    sql`SELECT 
      date,
      uniqMerge(dailyActiveUsers) as dailyActiveUsers
  FROM DailyActiveUsers
  GROUP BY date 
  HAVING dailyActiveUsers >= ${parseInt(minDailyActiveUsers)}
  ORDER BY date 
  LIMIT ${parseInt(limit)}`
  );
}
"#;

pub static BASE_AGGREGATION_TEMPLATE: &str = r#"
// This file is where you can define your SQL query for aggregating your data
// from other data models you have defined in Moose. For more information on the
// types of aggregate functions you can run on your existing data, consult the
// Clickhouse documentation: https://clickhouse.com/docs/en/sql-reference/aggregate-functions

import { Aggregation } from "@514labs/moose-lib";

export default {
  select: ``,
  orderBy: "",
} satisfies Aggregation as Aggregation;

"#;

pub static BASE_CONSUMPTION_TEMPLATE: &str = r#"
// This file is where you can define your API templates for consuming your data
// All query_params are passed in as strings, and are used within the sql tag to parameterize you queries
export interface QueryParams {
    
}
  
export default async function handle(
  {}: QueryParams,
  { client, sql }
) {

    return client.query(sql``);
}
"#;

pub static TS_BASE_MODEL_TEMPLATE: &str = r#"
// This file was auto-generated by the framework. You can add data models or change the existing ones

import { Key } from "@514labs/moose-lib";

export interface UserActivity {
    eventId: Key<string>;
    timestamp: string;
    userId: string;
    activity: string;
}

export interface ParsedActivity {
    eventId: Key<string>;
    timestamp: Date;
    userId: string;
    activity: string;
}

"#;

pub static VSCODE_EXTENSIONS_TEMPLATE: &str = r#"
{
    "recommendations": [
        "frigus02.vscode-sql-tagged-template-literals-syntax-only",
        "mtxr.sqltools",
        "ultram4rine.sqltools-clickhouse-driver",
        "jeppeandersen.vscode-kafka",
        "rangav.vscode-thunder-client"
    ]
}
"#;

pub static VSCODE_SETTINGS_TEMPLATE: &str = r#"
{
    "sqltools.connections": [
        {
            "server": "localhost",
            "port": 18123,
            "useHTTPS": false,
            "database": "local",
            "username": "panda",
            "enableTls": false,
            "password": "pandapass",
            "driver": "ClickHouse",
            "name": "moose clickhouse"
        }
    ]
}
"#;

// ==================================================================================================
// |                                          SDK TEMPLATES                                          |
// ==================================================================================================

pub fn render_package_json(package_name: &String) -> Result<String, TypescriptRenderingError> {
    let reg = Handlebars::new();

    let template_context = json!({
        "package_name": package_name
    });

    Ok(reg.render_template(PACKAGE_JSON_TEMPLATE, &template_context)?)
}

pub static PACKAGE_JSON_TEMPLATE: &str = r#"
{
    "name": "{{package_name}}",
    "version": "0.0.0",
    "description": "",
    "main": "dist/index.js",
    "types": "dist/index.d.ts",
    "scripts": {
        "build": "tsc --build",
        "clean": "tsc --build --clean"
      },
    "keywords": [],
    "author": "moose-cli",
    "license": "ISC",
    "devDependencies": {
        "@types/node": "^18.*.*",
        "typescript": "^5.*.*"
    },
    "dependencies": {}
}
"#;

pub fn render_ingest_client(
    version: &str,
    objects: &[TypescriptObjects],
) -> Result<String, TypescriptRenderingError> {
    let reg = Handlebars::new();

    let template_context = json!({
        "version": version,
        "version_with_underscore": version.replace('.', "_"),
        "ts_objects": objects.iter().map(|obj| {
            json!({
                "declaration_name": obj.interface.send_function_name(),
                "file_name": obj.interface.file_name().replace(".ts", ""),
                "var_name": obj.interface.var_name(),
                "interface_name": obj.interface.name,
                // TODO it is probably worth to have this be linked to infra object directly instead of setting
                // the path from a common convention
                "api_route": format!("ingest/{}", obj.interface.name),
            })
        }).collect::<Vec<Value>>()
    });

    Ok(reg.render_template(SDK_INGEST_INDEX_TEMPLATE, &template_context)?)
}

pub static SDK_INGEST_INDEX_TEMPLATE: &str = r#"

{{#each ts_objects}}
import { {{interface_name}} } from './{{file_name}}';
{{/each}}

// ==================================================================================================
// |      WARNING: This file is generated by the framework. Do NOT modify this file directly.        |
// ==================================================================================================

export class IngestClientV{{version_with_underscore}} {

    baseUrl: string;

    constructor(baseUrl: string) {
        this.baseUrl = baseUrl;
    }

    {{#each ts_objects}}
    async {{declaration_name}}({{var_name}}: {{interface_name}}) {
        return fetch(`${this.baseUrl}/{{api_route}}/{{../version}}`, {
            method: 'POST',
            mode: 'no-cors',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({{var_name}})
        })
    }
    
    {{/each}}
}

"#;

pub fn render_ts_config() -> Result<String, TypescriptRenderingError> {
    let reg = Handlebars::new();
    let template_context = json!({});
    Ok(reg.render_template(TS_CONFIG_TEMPLATE, &template_context)?)
}

// We're using the same pattern since we may want to allow the user to configure this in the future.
pub static TS_CONFIG_TEMPLATE: &str = r#"
{
    "compilerOptions": {
        "target": "ES2017",
        "module": "esnext",
        "moduleResolution": "node",
        "lib": ["es6"],
        "strict": true,
        "declaration": true,
        "removeComments": false,
        "outDir": "./dist",
    }
}
"#;

pub fn render_interface(
    interface: &TypescriptInterface,
) -> Result<String, TypescriptRenderingError> {
    let reg = Handlebars::new();

    let referred_types: Vec<&str> = interface
        .fields
        .iter()
        .filter_map(|field| {
            if let InterfaceFieldType::Object(inner) = &field.field_type {
                Some(inner.name.as_str())
            } else {
                None
            }
        })
        .collect();

    let template_context = json!({
        "interfaces_to_import": referred_types,
        "enums": interface.enums(),
        "name": interface.name,
        "fields": interface.fields.clone().into_iter().map(|field| {
            json!({
                "name": field.name,
                "field_type": field.field_type.to_string(),
                "is_optional": field.is_optional,
            })
        }).collect::<Vec<serde_json::Value>>()
    });

    Ok(reg.render_template(INTERFACE_TEMPLATE, &template_context)?)
}

pub static INTERFACE_TEMPLATE: &str = r#"
{{#unless (eq (len enums) 0)}} import { {{#each enums}}{{this}}{{#unless @last}}, {{/unless}}{{/each}} } from './enums'; {{/unless}}
{{#each interfaces_to_import}}import { {{this}} } from './{{this}}';
{{/each}}

// ==================================================================================================
// |      WARNING: This file is generated by the framework. Do NOT modify this file directly.        |
// ==================================================================================================

export interface {{name}} {
{{#each fields}}
    {{name}}{{#if is_optional}}?{{/if}}: {{field_type}};
{{/each}}
}
"#;

pub fn render_enums(ts_enum: HashSet<TSEnum>) -> Result<String, TypescriptRenderingError> {
    let reg = Handlebars::new();
    Ok(reg.render_template(ENUM_FILE_TEMPLATE, &json!({"enums": ts_enum}))?)
}

pub static ENUM_FILE_TEMPLATE: &str = r#"
// ==================================================================================================
// |      WARNING: This file is generated by the framework. Do NOT modify this file directly.        |
// ==================================================================================================

{{#each enums}}
export enum {{name}}{
    {{#each values}}
       {{name}}{{#if value}} ={{#if value.Number}} {{value.Number}}{{/if}}{{#if value.String }} "{{value.String}}"{{/if}}{{/if}},
    {{/each}}
}

{{/each}}
"#;
