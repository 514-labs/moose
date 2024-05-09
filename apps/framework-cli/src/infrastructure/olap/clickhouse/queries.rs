use crate::framework::controller::FrameworkObject;
use handlebars::Handlebars;
use serde_json::{json, Value};

use crate::framework::data_model::schema::EnumValue;
use crate::infrastructure::olap::clickhouse::model::{
    ClickHouseColumnType, ClickHouseFloat, ClickHouseInt, ClickHouseTable,
};
use crate::infrastructure::olap::clickhouse::version_sync::VersionSync;

use super::errors::ClickhouseError;
use super::model::ClickHouseColumn;

static CREATE_ALIAS_TEMPLATE: &str = r#"
CREATE VIEW IF NOT EXISTS {{db_name}}.{{alias_name}} AS SELECT * FROM {{db_name}}.{{source_table_name}};
"#;

fn create_alias_query(
    db_name: &str,
    alias_name: &str,
    source_table_name: &str,
) -> Result<String, ClickhouseError> {
    let reg = Handlebars::new();

    let context = json!({
        "db_name": db_name,
        "alias_name": alias_name,
        "source_table_name": source_table_name,
    });

    Ok(reg.render_template(CREATE_ALIAS_TEMPLATE, &context)?)
}

pub fn create_alias_query_from_table(
    old_table: &ClickHouseTable,
    new_table: &ClickHouseTable,
) -> Result<String, ClickhouseError> {
    create_alias_query(&old_table.db_name, &new_table.name, &old_table.name)
}

pub fn create_alias_query_from_framwork_object(
    latest_table: &FrameworkObject,
) -> Result<String, ClickhouseError> {
    create_alias_query(
        &latest_table.table.db_name,
        &latest_table.data_model.name,
        &latest_table.table.name,
    )
}

// TODO: Add column comment capability to the schema and template
static CREATE_TABLE_TEMPLATE: &str = r#"
CREATE TABLE IF NOT EXISTS {{db_name}}.{{table_name}} 
(
{{#each fields}}   {{field_name}} {{field_type}} {{field_nullable}}{{#unless @last}},{{/unless}} 
{{/each}}
)
ENGINE = {{engine}}
{{#if primary_key_string}}PRIMARY KEY ({{primary_key_string}}) {{/if}}
"#;

pub enum ClickhouseEngine {
    MergeTree,
}

pub fn create_table_query(
    table: ClickHouseTable,
    engine: ClickhouseEngine,
) -> Result<String, ClickhouseError> {
    let reg = Handlebars::new();

    let (engine, ignore_primary_key) = match engine {
        ClickhouseEngine::MergeTree => ("MergeTree".to_string(), false),
    };

    let primary_key = if ignore_primary_key {
        Vec::new()
    } else {
        table
            .columns
            .iter()
            .filter(|column| column.primary_key)
            .map(|column| column.name.clone())
            .collect::<Vec<String>>()
    };

    let template_context = json!({
        "db_name": table.db_name,
        "table_name": table.name,
        "fields":  builds_field_context(&table.columns)?,
        "primary_key_string": if !primary_key.is_empty() {
            Some(primary_key.join(", "))
        } else {
            None
        },
        "engine": engine
    });

    Ok(reg.render_template(CREATE_TABLE_TEMPLATE, &template_context)?)
}

static CREATE_VERSION_SYNC_TRIGGER_TEMPLATE: &str = r#"
CREATE MATERIALIZED VIEW IF NOT EXISTS {{db_name}}.{{view_name}} TO {{db_name}}.{{dest_table_name}}
(
    {{#each to_fields}} {{field_name}} {{field_type}} {{field_nullable}}{{#unless @last}},{{/unless}}
    {{/each}}
)
AS
SELECT
    {{#each to_fields}} moose_migrate_tuple.({{@index}} + 1) AS {{field_name}}{{#unless @last}},{{/unless}}
    {{/each}}
FROM (
    select {{migration_function_name}}(
        {{#each from_fields}} {{this}}{{#unless @last}},{{/unless}}
        {{/each}}
    ) as moose_migrate_tuple FROM {{db_name}}.{{source_table_name}}
)
"#;

pub fn create_version_sync_trigger_query(view: &VersionSync) -> Result<String, ClickhouseError> {
    let reg = Handlebars::new();

    let context = json!({
        "db_name": view.db_name,
        "view_name": view.migration_trigger_name(),
        "migration_function_name": view.migration_function_name(),
        "source_table_name": view.source_table.name,
        "dest_table_name": view.dest_table.name,
        "from_fields": view.source_table.columns.iter().map(|column| column.name.clone()).collect::<Vec<String>>(),
        "to_fields":  builds_field_context(&view.dest_table.columns)?,
    });

    Ok(reg.render_template(CREATE_VERSION_SYNC_TRIGGER_TEMPLATE, &context)?)
}

static INITIAL_DATA_LOAD_TEMPLATE: &str = r#"
INSERT INTO {{db_name}}.{{dest_table_name}}
SELECT
    {{#each to_fields}} moose_migrate_tuple.({{@index}} + 1) AS {{field_name}}{{#unless @last}},{{/unless}}
    {{/each}}
FROM (
    select {{migration_function_name}}(
        {{#each from_fields}}{{this}} {{#unless @last}},{{/unless}}
        {{/each}}
    ) as moose_migrate_tuple FROM {{db_name}}.{{source_table_name}}
)
"#;

pub fn create_initial_data_load_query(view: &VersionSync) -> Result<String, ClickhouseError> {
    let reg = Handlebars::new();

    let context = json!({
        "db_name": view.db_name,
        "dest_table_name": view.dest_table.name,
        "migration_function_name": view.migration_function_name(),
        "source_table_name": view.source_table.name,
        "from_fields": view.source_table.columns.iter().map(|column| column.name.clone()).collect::<Vec<String>>(),
        "to_fields": builds_field_context(&view.dest_table.columns)?,
    });

    Ok(reg.render_template(INITIAL_DATA_LOAD_TEMPLATE, &context)?)
}

pub static DROP_TABLE_TEMPLATE: &str = r#"
DROP TABLE IF EXISTS {{db_name}}.{{table_name}};
"#;

pub fn drop_table_query(table: ClickHouseTable) -> Result<String, ClickhouseError> {
    let reg = Handlebars::new();

    let context = json!({
        "db_name": table.db_name,
        "table_name": table.name,
    });

    Ok(reg.render_template(DROP_TABLE_TEMPLATE, &context)?)
}

fn field_type_to_string(field_type: &ClickHouseColumnType) -> Result<String, ClickhouseError> {
    // Blowing out match statements here in case we need to customize the output string for some types.
    match field_type {
        ClickHouseColumnType::String => Ok(field_type.to_string()),
        ClickHouseColumnType::Boolean => Ok(field_type.to_string()),
        ClickHouseColumnType::ClickhouseInt(int) => match int {
            ClickHouseInt::Int8 => Ok(int.to_string()),
            ClickHouseInt::Int16 => Ok(int.to_string()),
            ClickHouseInt::Int32 => Ok(int.to_string()),
            ClickHouseInt::Int64 => Ok(int.to_string()),
            ClickHouseInt::Int128 => Ok(int.to_string()),
            ClickHouseInt::Int256 => Ok(int.to_string()),
            ClickHouseInt::UInt8 => Ok(int.to_string()),
            ClickHouseInt::UInt16 => Ok(int.to_string()),
            ClickHouseInt::UInt32 => Ok(int.to_string()),
            ClickHouseInt::UInt64 => Ok(int.to_string()),
            ClickHouseInt::UInt128 => Ok(int.to_string()),
            ClickHouseInt::UInt256 => Ok(int.to_string()),
        },
        ClickHouseColumnType::ClickhouseFloat(float) => match float {
            ClickHouseFloat::Float32 => Ok(float.to_string()),
            ClickHouseFloat::Float64 => Ok(float.to_string()),
        },
        ClickHouseColumnType::Decimal => Ok(field_type.to_string()),
        ClickHouseColumnType::DateTime => Ok(field_type.to_string()),
        ClickHouseColumnType::Enum(data_enum) => {
            let enum_statement = data_enum
                .values
                .iter()
                .map(|enum_member| match &enum_member.value {
                    EnumValue::Int(int) => format!("'{}' = {}", enum_member.name, int),
                    EnumValue::String(string) => format!("'{}'", string),
                })
                .collect::<Vec<String>>()
                .join(",");

            Ok(format!("Enum({})", enum_statement))
        }
        ClickHouseColumnType::Json => Err(ClickhouseError::UnsupportedDataType {
            type_name: "Json".to_string(),
        }),
        ClickHouseColumnType::Bytes => Err(ClickhouseError::UnsupportedDataType {
            type_name: "Bytes".to_string(),
        }),
        ClickHouseColumnType::Array(inner_type) => {
            let inner_type_string = field_type_to_string(inner_type)?;
            Ok(format!("Array({})", inner_type_string))
        }
    }
}

fn builds_field_context(columns: &[ClickHouseColumn]) -> Result<Vec<Value>, ClickhouseError> {
    columns
        .iter()
        .map(|column| {
            let field_type = field_type_to_string(&column.column_type)?;

            Ok(json!({
                "field_name": column.name,
                "field_type": field_type,
                // Clickhouse doesn't allow array fields to be nullable
                "field_nullable": if column.required || column.is_array() {
                    "NOT NULL".to_string()
                } else {
                    "NULL".to_string()
                },
            }))
        })
        .collect::<Result<Vec<Value>, ClickhouseError>>()
}
