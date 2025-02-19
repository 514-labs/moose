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

pub static TS_BASE_STREAMING_FUNCTION_SAMPLE: &str = r#"
// Example streaming function: Reshapes Foo data to Bar format.

// Imports: Source (Foo) and Destination (Bar) data models.
import { Foo, Bar } from "datamodels/models";


// The 'run' function contains the logic that runs on each new data point in the Foo stream.
// For more details on how Moose streaming functions work, see: https://docs.moosejs.com
export default function run(foo: Foo): Bar {
  return {
    primaryKey: foo.primaryKey,
    utcTimestamp: new Date(foo.timestamp),
    textLength: foo.optionalText?.length ?? 0,
    hasText: foo.optionalText !== null,
  } as Bar;
}

"#;

pub static TS_BASE_STREAMING_FUNCTION_TEMPLATE: &str = r#"
// Import your Moose data models to use in the streaming function
{{source_import}}
{{destination_import}}

// The 'run' function transforms {{source}} data to {{destination}} format.
// For more details on how Moose streaming functions work, see: https://docs.moosejs.com
export default function run(source: {{source}}): {{destination}} | null {
  return {{destination_object}};
}

"#;

pub static TS_BASE_BLOCKS_SAMPLE: &str = r#"
// Example Block that creates a materialized view that aggregates daily statistics from Bar_0_0

import { Blocks } from "@514labs/moose-lib"; // Import Blocks to structure setup/teardown queries

const MV_NAME = "BarAggregated_MV";

const MV_QUERY = `
CREATE MATERIALIZED VIEW ${MV_NAME}
ENGINE = MergeTree()
ORDER BY dayOfMonth
POPULATE
AS
SELECT
  toDayOfMonth(utcTimestamp) as dayOfMonth,
  count(primaryKey) as totalRows,
  countIf(hasText) as rowsWithText,
  sum(textLength) as totalTextLength,
  max(textLength) as maxTextLength
FROM Bar_0_0
GROUP BY dayOfMonth
`;

const DROP_MV_QUERY = `
DROP TABLE IF EXISTS ${MV_NAME}
`;

export default {
  teardown: [DROP_MV_QUERY], // SQL to drop the materialized view if it exists
  setup: [MV_QUERY], // SQL to create a materialized view that aggregates daily statistics from Bar_0_0
} as Blocks;

"#;

pub static TS_BASE_APIS_SAMPLE: &str = r#"
import {
  createConsumptionApi,
  ConsumptionHelpers as CH,
} from "@514labs/moose-lib";
import { tags } from "typia";

// This file is where you can define your APIs to consume your data
interface QueryParams {
  orderBy: "totalRows" | "rowsWithText" | "maxTextLength" | "totalTextLength";
  limit?: number;
  startDay?: number & tags.Type<"int32"> & tags.Minimum<1> & tags.Maximum<31>;
  endDay?: number & tags.Type<"int32"> & tags.Minimum<1> & tags.Maximum<31>;
}

// createConsumptionApi uses compile time code generation to generate a parser for QueryParams
export default createConsumptionApi<QueryParams>(
  async (
    { orderBy = "totalRows", limit = 5, startDay = 1, endDay = 31 },
    { client, sql }
  ) => {
    const query = sql`
      SELECT 
        dayOfMonth,
        ${CH.column(orderBy)}
      FROM BarAggregated_MV
      WHERE 
        dayOfMonth >= ${startDay} 
        AND dayOfMonth <= ${endDay}
      ORDER BY ${CH.column(orderBy)} DESC
      LIMIT ${limit}
    `;

    const data = await client.query<{
      dayOfMonth: number;
      totalRows?: number;
      rowsWithText?: number;
      maxTextLength?: number;
      totalTextLength?: number;
    }>(query);

    return data;
  }
);
"#;

pub static TS_BASE_BLOCK_TEMPLATE: &str = r#"
// This file is where you can define your SQL queries to shape and manipulate batches
// of data using Blocks. There is a collection of helper functions to create SQL queries
// within the @514labs/moose-lib package. 

// Blocks can also manage materialized views to store the results of your queries for 
// improved performance. A materialized view is the recommended approach for aggregating
// data. For more information on the types of aggregate functions you can run on your existing data, 
// consult the Clickhouse documentation: https://clickhouse.com/docs/en/sql-reference/aggregate-functions

import { Blocks } from "@514labs/moose-lib";

export default {
  teardown: [],
  setup: [],
} as Blocks;
"#;

pub static TS_BASE_CONSUMPTION_TEMPLATE: &str = r#"
import { createConsumptionApi } from "@514labs/moose-lib";

// This file is where you can define your API templates for consuming your data
interface QueryParams {}

// createConsumptionApi uses compile time code generation to generate a parser for QueryParams
export default createConsumptionApi<QueryParams>(
  async (params, { client, sql }) => {
    return client.query(sql`SELECT 1`);
  }
);
"#;

pub static TS_BASE_MODEL_TEMPLATE: &str = r#"
// This file was auto-generated by the framework. You can add data models or change the existing ones

import { Key } from "@514labs/moose-lib";

export interface Foo {
    primaryKey: Key<string>;
    timestamp: number;
    optionalText?: string;
}

export interface Bar {
    primaryKey: Key<string>;
    utcTimestamp: Date;
    hasText: boolean;
    textLength: number;
}

"#;

pub static TS_BASE_SCRIPT_TEMPLATE: &str = r#"import { TaskFunction, TaskDefinition } from "@514labs/moose-lib";

const {{name}}: TaskFunction = async () => {
    // The body of your script goes here
    console.log("Hello world from {{name}}");

    // The return value is the output of the script.
    // The return value should be a dictionary with at least:
    // - step: the step name (e.g., "extract", "transform")
    // - data: the actual data being passed to the next step
    return {
        step: "{{name}}",
        data: {}
    };
};

export default function createTask(input?: object) {
    return {
        task: {{name}},
        config: {
            retries: 3,
        }
    } as TaskDefinition;
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
    ]{language_specific_settings}
}
"#;

pub static VS_CODE_PYTHON_SETTINGS: &str = r#",
    "python.analysis.extraPaths": [".moose/versions"],
    "python.analysis.typeCheckingMode": "basic""#;

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
