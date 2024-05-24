use serde::Serialize;

use handlebars::Handlebars;
use serde_json::json;
use tinytemplate::TinyTemplate;

use super::generator::{
    InterfaceField, TSEnum, TypescriptInterface, TypescriptObjects, TypescriptPackage,
};

#[derive(Debug, thiserror::Error)]
#[error("Failed to generate Typescript code")]
#[non_exhaustive]
pub enum TypescriptRenderingError {
    HandlebarError(#[from] handlebars::RenderError),
}

pub static INTERFACE_TEMPLATE: &str = r#"
{{#if has_enums}} import * from './enums'; {{/if}}

// ==================================================================================================
// |      WARNING: This file is generated by the framework. Do NOT modify this file directly.        |
// | This file will be removed in future releases that will use typescript as a modeling language.   |
// ==================================================================================================

export interface {{name}} {
{{#each fields}}
    {{name}}{{#if is_optional}}?{{/if}}: {{field_type}};
{{/each}}
}
"#;

pub fn render_interface(
    interface: &TypescriptInterface,
) -> Result<String, TypescriptRenderingError> {
    let reg = Handlebars::new();

    let template_context = json!({
        "has_enums": interface.has_enums(),
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

#[derive(Serialize)]
struct InterfaceContext {
    name: String,
    file_name: String,
    var_name: String,
    fields: Vec<InterfaceFieldContext>,
}

impl InterfaceContext {
    fn new(interface: &TypescriptInterface) -> InterfaceContext {
        InterfaceContext {
            name: interface.name.clone(),
            file_name: interface.file_name(),
            var_name: interface.var_name(),
            fields: interface
                .fields
                .clone()
                .into_iter()
                .map(InterfaceFieldContext::new)
                .collect::<Vec<InterfaceFieldContext>>(),
        }
    }
}

#[derive(Serialize)]
struct InterfaceFieldContext {
    name: String,
    field_type: String,
    is_optional: bool,
}

impl InterfaceFieldContext {
    fn new(interface_field: InterfaceField) -> InterfaceFieldContext {
        InterfaceFieldContext {
            name: interface_field.name,
            field_type: interface_field.field_type.to_string(),
            is_optional: interface_field.is_optional,
        }
    }
}

pub static SEND_FUNC_TEMPLATE: &str = r#"
// ==================================================================================================
// |      WARNING: This file is generated by the framework. Do NOT modify this file directly.        |
// ==================================================================================================

import \{ {interface_context.name} } from './{interface_context.file_name}';

export async function {declaration_name}({interface_context.var_name}: {interface_context.name}) \{
    return fetch('{server_url}/{api_route_name}', \{
        method: 'POST',
        mode: 'no-cors',
        headers: \{
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({interface_context.var_name})
    })
}
"#;

#[derive(Serialize)]
pub struct SendFunctionContext {
    interface_context: InterfaceContext,
    declaration_name: String,
    file_name: String,
    server_url: String,
    api_route_name: String,
}

impl SendFunctionContext {
    fn new(
        interface: &TypescriptInterface,
        server_url: String,
        api_route_name: String,
    ) -> SendFunctionContext {
        SendFunctionContext {
            interface_context: InterfaceContext::new(interface),
            declaration_name: interface.send_function_name(),
            file_name: interface.send_function_file_name(),
            server_url,
            api_route_name,
        }
    }
}

pub struct SendFunctionTemplate;

impl SendFunctionTemplate {
    pub fn build(
        interface: &TypescriptInterface,
        server_url: String,
        api_route_name: String,
    ) -> String {
        let mut tt = TinyTemplate::new();
        tt.add_template("send", SEND_FUNC_TEMPLATE).unwrap();
        let context = SendFunctionContext::new(interface, server_url, api_route_name);

        tt.render("send", &context).unwrap()
    }
}

pub static INDEX_TEMPLATE: &str = r#"

// ==================================================================================================
// |      WARNING: This file is generated by the framework. Do NOT modify this file directly.        |
// ==================================================================================================
{{- for ts_object in ts_objects}}
import \{ {ts_object.interface_context.name} } from './{latest_version}/{ts_object.interface_context.name}';
import \{ {ts_object.send_function_context.declaration_name} } from './{latest_version}/{ts_object.send_function_context.file_name}';
{{endfor}}

{{for ts_object in ts_objects}}

export \{ {ts_object.interface_context.name} };
export \{ {ts_object.send_function_context.declaration_name} };
{{- endfor}}
"#;

#[derive(Serialize)]
struct TypescriptObjectsContext {
    interface_context: InterfaceContext,
    send_function_context: SendFunctionContext,
}

impl TypescriptObjectsContext {
    fn new(ts_objects: &TypescriptObjects) -> TypescriptObjectsContext {
        TypescriptObjectsContext {
            interface_context: InterfaceContext::new(&ts_objects.interface),
            send_function_context: SendFunctionContext::new(
                &ts_objects.interface,
                ts_objects.send_function.server_url.clone(),
                ts_objects.send_function.api_route_name.clone(),
            ),
        }
    }
}

#[derive(Serialize)]
struct IndexContext {
    latest_version: String,
    ts_objects: Vec<TypescriptObjectsContext>,
}
impl IndexContext {
    fn new(latest_version: String, ts_objects: &[TypescriptObjects]) -> IndexContext {
        IndexContext {
            latest_version,
            ts_objects: ts_objects
                .iter()
                .map(TypescriptObjectsContext::new)
                .collect::<Vec<TypescriptObjectsContext>>(),
        }
    }
}
pub struct IndexTemplate;

impl IndexTemplate {
    pub fn build(latest_version: &str, latest_objects: &[TypescriptObjects]) -> String {
        let mut tt = TinyTemplate::new();
        tt.add_template("index", INDEX_TEMPLATE).unwrap();
        let context = IndexContext::new(latest_version.to_string(), latest_objects);

        tt.render("index", &context).unwrap()
    }
}

pub static PACKAGE_JSON_TEMPLATE: &str = r#"
\{
    "name": "{package_name}",
    "version": "0.0.0",
    "description": "",
    "main": "dist/index.js",
    "types": "dist/index.d.ts",
    "scripts": \{
        "build": "tsc --build",
        "clean": "tsc --build --clean"
      },
    "keywords": [],
    "author": "moose-cli",
    "license": "ISC",
    "devDependencies": \{
        "@types/node": "^18.*.*",
        "typescript": "^5.*.*"
    },
    "dependencies": \{

    }
}
"#;

#[derive(Serialize)]
pub struct PackageJsonContext {
    package_name: String,
    // package_version: String,
    // package_author: String,
}

impl PackageJsonContext {
    fn new(package_name: String) -> PackageJsonContext {
        PackageJsonContext {
            package_name,
            // package_version,
            // package_author,
        }
    }
}

pub struct PackageJsonTemplate;

impl PackageJsonTemplate {
    pub fn build(package: &TypescriptPackage) -> String {
        let mut tt = TinyTemplate::new();
        tt.add_template("package_json", PACKAGE_JSON_TEMPLATE)
            .unwrap();
        let context = PackageJsonContext::new(package.name.clone());

        tt.render("package_json", &context).unwrap()
    }
}

// I'm using the same pattern since we may want to allow the user to configure this in the future.
pub static TS_CONFIG_TEMPLATE: &str = r#"
\{
    "compilerOptions": \{
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

#[derive(Serialize)]
pub struct TsConfigContext;

impl TsConfigContext {
    fn new() -> TsConfigContext {
        TsConfigContext
    }
}

pub struct TsConfigTemplate;

impl TsConfigTemplate {
    pub fn build() -> String {
        let mut tt = TinyTemplate::new();
        tt.add_template("ts_config", TS_CONFIG_TEMPLATE).unwrap();
        let context = TsConfigContext::new();

        tt.render("ts_config", &context).unwrap()
    }
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

pub fn render_enums(ts_enum: Vec<TSEnum>) -> Result<String, TypescriptRenderingError> {
    let reg = Handlebars::new();
    Ok(reg.render_template(ENUM_FILE_TEMPLATE, &json!({"enums": ts_enum}))?)
}

pub static BASE_FLOW_SAMPLE_TEMPLATE: &str = r#"
// Example flow function: Converts local timestamps in UserActivity data to UTC.

// Imports: Source (UserActivity) and Destination (ParsedActivity) data models.
import { ParsedActivity, UserActivity } from "../../../datamodels/models.ts";

// The 'run' function transforms UserActivity data to ParsedActivity format.
// For more details on how Moose flows work, see: https://docs.moosejs.com
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

pub static BASE_FLOW_TEMPLATE: &str = r#"
// Add your models & start the development server to import these types
{{imports}}

// The 'run' function transforms {{source}} data to {{destination}} format.
// For more details on how Moose flows work, see: https://docs.moosejs.com
export default function run(source: {{source}}): {{destination}} | null {
  return {{destination_object}};
}

"#;

pub static BASE_AGGREGATION_SAMPLE_TEMPLATE: &str = r#"
// Here is a sample aggregation query that calculates the number of daily active users
// based on the number of unique users who complete a sign-in activity each day.

interface Aggregation {
  select: string;
  orderBy: string;
}

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

pub static BASE_APIS_SAMPLE_TEMPLATE: &str = r#"
// Here is a sample api configuration that creates an API which serves the daily active users materialized view

interface QueryParams {
  limit: string;
  minDailyActiveUsers: string;
}

export default async function handle(
  { limit = "10", minDailyActiveUsers = "0" }: QueryParams,
  { client, sql }
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

interface Aggregation {
  select: string;
  orderBy: string;
}

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
    
      return client.query(sql``
    );
  }
  
"#;

pub static BASE_MODEL_TEMPLATE: &str = r#"
// This file was auto-generated by the framework. You can add data models or change the existing ones

type Key<T extends string | number> = T

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
        "ultram4rine.sqltools-clickhouse-driver"
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
