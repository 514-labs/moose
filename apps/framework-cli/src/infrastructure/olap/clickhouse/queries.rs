use crate::framework::controller::FrameworkObject;
use serde::Serialize;
use tinytemplate::{format_unescaped, TinyTemplate};

use crate::framework::schema::EnumValue;
use crate::infrastructure::olap::clickhouse::model::{
    ClickHouseColumn, ClickHouseColumnType, ClickHouseFloat, ClickHouseInt, ClickHouseTable,
};
use crate::infrastructure::olap::clickhouse::version_sync::VersionSync;

use super::errors::ClickhouseError;
use super::QueryString;

static CREATE_ALIAS_TEMPLATE: &str = r#"
CREATE VIEW IF NOT EXISTS {db_name}.{alias_name} AS SELECT * FROM {db_name}.{source_table_name};
"#;

// TODO: Add column comment capability to the schema and template
static CREATE_TABLE_TEMPLATE: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.{table_name} 
(
{{for field in fields}}{field.field_name} {field.field_type} {field.field_nullable},
{{endfor}}
{{if primary_key_string}}
PRIMARY KEY ({primary_key_string})
{{endif}}
)
ENGINE = {engine};
"#;

static CREATE_VERSION_SYNC_TRIGGER_TEMPLATE: &str = r#"
CREATE MATERIALIZED VIEW IF NOT EXISTS {db_name}.{view_name} TO {db_name}.{dest_table_name}
(
{{for field in to_fields}}{field.field_name} {field.field_type} {field.field_nullable}{{- if @last }}{{ else }}, {{ endif }}
{{endfor}}
)
AS
SELECT
{{for field in to_fields}} moose_migrate_tuple.({@index} + 1) AS {field.field_name}{{- if @last }}{{ else }}, {{ endif }}
{{endfor}}
FROM (select {migration_function_name}(
{{for field in from_fields}}{field}{{- if @last }}{{ else }}, {{ endif }}
{{endfor}}
) as moose_migrate_tuple FROM {db_name}.{source_table_name})
"#;

static INITIAL_DATA_LOAD_TEMPLATE: &str = r#"
INSERT INTO {db_name}.{dest_table_name}
SELECT
{{for field in to_fields}} moose_migrate_tuple.({@index} + 1) AS {field.field_name}{{- if @last }}{{ else }}, {{ endif }}
{{endfor}}
FROM (select {migration_function_name}(
{{for field in from_fields}}{field}{{- if @last }}{{ else }}, {{ endif }}
{{endfor}}
) as moose_migrate_tuple FROM {db_name}.{source_table_name})
"#;

pub struct CreateAliasQuery;
impl CreateAliasQuery {
    fn render(context: CreateAliasContext) -> String {
        let mut tt = TinyTemplate::new();
        tt.add_template("create_alias", CREATE_ALIAS_TEMPLATE)
            .unwrap();
        tt.render("create_alias", &context).unwrap()
    }

    pub fn build(old_table: &ClickHouseTable, new_table: &ClickHouseTable) -> String {
        CreateAliasQuery::render(CreateAliasContext {
            db_name: old_table.db_name.clone(),
            alias_name: new_table.name.clone(),
            source_table_name: old_table.name.clone(),
        })
    }

    pub fn build_latest(latest_table: &FrameworkObject) -> String {
        CreateAliasQuery::render(CreateAliasContext {
            db_name: latest_table.table.db_name.clone(),
            alias_name: latest_table.data_model.name.clone(),
            source_table_name: latest_table.table.name.clone(),
        })
    }
}
#[derive(Serialize)]
struct CreateAliasContext {
    db_name: String,
    alias_name: String,
    source_table_name: String,
}

pub struct CreateTableQuery;

pub enum ClickhouseEngine {
    MergeTree,
}

impl CreateTableQuery {
    pub fn build(
        table: ClickHouseTable,
        engine: ClickhouseEngine,
    ) -> Result<String, ClickhouseError> {
        let mut tt = TinyTemplate::new();
        tt.set_default_formatter(&format_unescaped); // by default it formats HTML-escaped and messes up single quotes
        tt.add_template("create_table", CREATE_TABLE_TEMPLATE)
            .unwrap();
        let context = CreateTableContext::new(table, engine)?;
        let rendered = tt.render("create_table", &context).unwrap();
        Ok(rendered)
    }
}

#[derive(Serialize)]
struct CreateTableContext {
    db_name: String,
    table_name: String,
    fields: Vec<CreateTableFieldContext>,
    primary_key_string: Option<String>,
    engine: String,
}

impl CreateTableContext {
    fn new(
        table: ClickHouseTable,
        engine: ClickhouseEngine,
    ) -> Result<CreateTableContext, ClickhouseError> {
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

        Ok(CreateTableContext {
            db_name: table.db_name,
            table_name: table.name,
            fields: table
                .columns
                .into_iter()
                .map(CreateTableFieldContext::new)
                .collect::<Result<Vec<CreateTableFieldContext>, ClickhouseError>>()?,
            primary_key_string: if !primary_key.is_empty() {
                Some(primary_key.join(", "))
            } else {
                None
            },
            engine,
        })
    }
}

#[derive(Serialize)]
struct CreateTableFieldContext {
    field_name: String,
    field_type: String,
    field_nullable: String,
}

impl CreateTableFieldContext {
    fn new(column: ClickHouseColumn) -> Result<CreateTableFieldContext, ClickhouseError> {
        clickhouse_column_to_create_table_field_context(column)
    }
}

pub static DROP_TABLE_TEMPLATE: &str = r#"
DROP TABLE IF EXISTS {db_name}.{table_name};
"#;

pub struct DropTableQuery;

impl DropTableQuery {
    pub fn build(table: ClickHouseTable) -> Result<String, ClickhouseError> {
        let mut tt = TinyTemplate::new();
        tt.add_template("drop_table", DROP_TABLE_TEMPLATE).unwrap();
        let context = DropTableContext::new(table)?;
        let rendered = tt.render("drop_table", &context).unwrap();
        Ok(rendered)
    }
}

#[derive(Serialize)]
struct DropTableContext {
    db_name: String,
    table_name: String,
}

impl DropTableContext {
    fn new(table: ClickHouseTable) -> Result<DropTableContext, ClickhouseError> {
        Ok(DropTableContext {
            db_name: table.db_name,
            table_name: table.name,
        })
    }
}

pub struct InitialLoadQuery;
impl InitialLoadQuery {
    pub fn build(view: VersionSync) -> Result<QueryString, ClickhouseError> {
        let mut tt = TinyTemplate::new();
        tt.add_template("initial_load_trigger", INITIAL_DATA_LOAD_TEMPLATE)
            .unwrap();
        // same field names as the trigger context
        let context = CreateVersionSyncTriggerContext::new(&view)?;
        Ok(tt.render("initial_load_trigger", &context).unwrap())
    }
}

pub struct CreateVersionSyncTriggerQuery;
impl CreateVersionSyncTriggerQuery {
    pub fn build(view: &VersionSync) -> Result<QueryString, ClickhouseError> {
        let mut tt = TinyTemplate::new();
        tt.add_template(
            "create_version_sync_trigger",
            CREATE_VERSION_SYNC_TRIGGER_TEMPLATE,
        )
        .unwrap();
        let context = CreateVersionSyncTriggerContext::new(view)?;
        Ok(tt.render("create_version_sync_trigger", &context).unwrap())
    }
}

#[derive(Serialize)]
struct CreateVersionSyncTriggerContext {
    db_name: String,
    view_name: String,
    migration_function_name: String,
    source_table_name: String,
    dest_table_name: String,
    from_fields: Vec<String>,
    to_fields: Vec<CreateTableFieldContext>,
}

impl CreateVersionSyncTriggerContext {
    pub fn new(
        version_sync: &VersionSync,
    ) -> Result<CreateVersionSyncTriggerContext, ClickhouseError> {
        let trigger_name = version_sync.migration_trigger_name();
        let migration_function_name = version_sync.migration_function_name();

        Ok(CreateVersionSyncTriggerContext {
            db_name: version_sync.db_name.clone(),
            view_name: trigger_name,
            migration_function_name,
            source_table_name: version_sync.source_table.name.clone(),
            dest_table_name: version_sync.dest_table.name.clone(),
            from_fields: version_sync
                .source_table
                .columns
                .iter()
                .map(|column| column.name.clone())
                .collect(),

            to_fields: version_sync
                .dest_table
                .columns
                .iter()
                .map(|c| CreateTableFieldContext::new(c.clone()))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

fn field_type_to_string(field_type: ClickHouseColumnType) -> Result<String, ClickhouseError> {
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
                    Some(value) => match value {
                        EnumValue::Int(int) => format!("'{}' = {}", enum_member.name, int),
                        EnumValue::String(string) => format!("'{}'", string),
                    },
                    None => {
                        format!("'{}'", enum_member.name)
                    }
                })
                .collect::<Vec<String>>()
                .join(",");

            Ok(format!("Enum({})", enum_statement))
        }
        ClickHouseColumnType::Json => Err(ClickhouseError::UnsupportedDataTypeError {
            type_name: "Json".to_string(),
        }),
        ClickHouseColumnType::Bytes => Err(ClickhouseError::UnsupportedDataTypeError {
            type_name: "Bytes".to_string(),
        }),
        ClickHouseColumnType::Array(inner_type) => {
            let inner_type_string = field_type_to_string(*inner_type)?;
            Ok(format!("Array({})", inner_type_string))
        }
    }
}

fn clickhouse_column_to_create_table_field_context(
    column: ClickHouseColumn,
) -> Result<CreateTableFieldContext, ClickhouseError> {
    let field_type = field_type_to_string(column.column_type)?;
    Ok(CreateTableFieldContext {
        field_name: column.name,
        field_type,
        field_nullable: if column.required {
            "NOT NULL".to_string()
        } else {
            "NULL".to_string()
        },
    })
}

#[cfg(test)]
mod tests {
    use crate::framework::{controller::framework_object_mapper, schema::parse_data_model_file};

    #[test]
    fn test_create_query_from_prisma_model() {
        let current_dir = std::env::current_dir().unwrap();

        let test_file = current_dir.join("tests/psl/simple.prisma");

        let result = parse_data_model_file(&test_file, "1.0", framework_object_mapper).unwrap();

        let ch_table = result[0].table.clone();

        let query = ch_table.create_data_table_query().unwrap();

        let expected = r#"
CREATE TABLE IF NOT EXISTS local.User_1_0 
(
id Int64 NOT NULL,
email String NOT NULL,
name String NULL,


PRIMARY KEY (id)

)
ENGINE = MergeTree;
"#;

        assert_eq!(query, expected);
    }
}
