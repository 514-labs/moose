use schema_ast::ast::FieldArity;
use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::{
    framework::schema::UnsupportedDataTypeError,
    infrastructure::olap::clickhouse::{
        ClickhouseColumn, ClickhouseColumnType, ClickhouseFloat, ClickhouseInt, ClickhouseTable,
    },
};

use super::ClickhouseView;

// TODO: Add column comment capability to the schemna and template
pub static CREATE_TABLE_TEMPLATE: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.{table_name} 
(
{{for field in fields}}{field.field_name} {field.field_type} {field.field_arity},
{{endfor}}
{{if primary_key_string}}
PRIMARY KEY ({primary_key_string})
{{endif}}
)
ENGINE = Kafka('{cluster_network}:{kafka_port}', '{topic}', 'clickhouse-group', 'JSONEachRow');
"#;

pub struct CreateTableQuery;

impl CreateTableQuery {
    pub fn build(
        table: ClickhouseTable,
        cluster_network: String,
        kafka_port: u16,
        topic: String,
    ) -> Result<String, UnsupportedDataTypeError> {
        let mut tt = TinyTemplate::new();
        tt.add_template("create_table", CREATE_TABLE_TEMPLATE)
            .unwrap();
        let context = CreateTableContext::new(table, cluster_network, kafka_port, topic)?;
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
    cluster_network: String,
    kafka_port: u16,
    topic: String,
}

impl CreateTableContext {
    fn new(
        table: ClickhouseTable,
        cluster_network: String,
        kafka_port: u16,
        topic: String,
    ) -> Result<CreateTableContext, UnsupportedDataTypeError> {
        let primary_key = table
            .columns
            .iter()
            .filter(|column| column.primary_key)
            .map(|column| column.name.clone())
            .collect::<Vec<String>>();

        Ok(CreateTableContext {
            db_name: table.db_name,
            table_name: table.name,
            fields: table
                .columns
                .into_iter()
                .map(CreateTableFieldContext::new)
                .collect::<Result<Vec<CreateTableFieldContext>, UnsupportedDataTypeError>>()?,
            primary_key_string: if !primary_key.is_empty() {
                Some(primary_key.join(", "))
            } else {
                None
            },
            cluster_network,
            kafka_port,
            topic,
        })
    }
}

#[derive(Serialize)]
struct CreateTableFieldContext {
    field_name: String,
    field_type: String,
    field_arity: String,
}

impl CreateTableFieldContext {
    fn new(column: ClickhouseColumn) -> Result<CreateTableFieldContext, UnsupportedDataTypeError> {
        clickhouse_column_to_create_table_field_context(column)
    }
}

pub static DROP_TABLE_TEMPLATE: &str = r#"
DROP TABLE IF EXISTS {db_name}.{table_name};
"#;

pub struct DropTableQuery;

impl DropTableQuery {
    pub fn build(table: ClickhouseTable) -> Result<String, UnsupportedDataTypeError> {
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
    fn new(table: ClickhouseTable) -> Result<DropTableContext, UnsupportedDataTypeError> {
        Ok(DropTableContext {
            db_name: table.db_name,
            table_name: table.name,
        })
    }
}

pub static CREATE_MATERIALIZED_VIEW_TEMPLATE: &str = r#"
CREATE MATERIALIZED VIEW IF NOT EXISTS {db_name}.{view_name} 
ENGINE = Memory
AS
SELECT * FROM {db_name}.{source_table_name}
SETTINGS
stream_like_engine_allow_direct_select = 1;
"#;

pub struct CreateMaterializedViewQuery;

impl CreateMaterializedViewQuery {
    pub fn build(view: ClickhouseView) -> Result<String, UnsupportedDataTypeError> {
        let mut tt = TinyTemplate::new();
        tt.add_template(
            "create_materialized_view",
            CREATE_MATERIALIZED_VIEW_TEMPLATE,
        )
        .unwrap();
        let context = CreateMaterializedViewContext::new(view)?;
        let rendered = tt.render("create_materialized_view", &context).unwrap();
        Ok(rendered)
    }
}

pub static DROP_MATERIALIZED_VIEW_TEMPLATE: &str = r#"
DROP TABLE IF EXISTS {db_name}.{view_name};
"#;

pub struct DropMaterializedViewQuery;

impl DropMaterializedViewQuery {
    pub fn build(table: ClickhouseView) -> Result<String, UnsupportedDataTypeError> {
        let mut tt = TinyTemplate::new();
        tt.add_template("drop_materialized_view", DROP_MATERIALIZED_VIEW_TEMPLATE)
            .unwrap();
        let context = DropMaterializedViewContext::new(table)?;
        let rendered = tt.render("drop_materialized_view", &context).unwrap();
        Ok(rendered)
    }
}

#[derive(Serialize)]
struct DropMaterializedViewContext {
    db_name: String,
    view_name: String,
}

impl DropMaterializedViewContext {
    fn new(view: ClickhouseView) -> Result<DropMaterializedViewContext, UnsupportedDataTypeError> {
        Ok(DropMaterializedViewContext {
            db_name: view.db_name,
            view_name: format!("{}_view", view.name),
        })
    }
}

#[derive(Serialize)]
struct CreateMaterializedViewContext {
    db_name: String,
    view_name: String,
    source_table_name: String,
}

impl CreateMaterializedViewContext {
    fn new(
        view: ClickhouseView,
    ) -> Result<CreateMaterializedViewContext, UnsupportedDataTypeError> {
        Ok(CreateMaterializedViewContext {
            db_name: view.db_name,
            view_name: view.name,
            source_table_name: view.source_table.name,
        })
    }
}

fn field_type_to_string(
    field_type: ClickhouseColumnType,
) -> Result<String, UnsupportedDataTypeError> {
    // Blowing out match statements here in case we need to customize the output string for some types.
    match field_type {
        ClickhouseColumnType::String => Ok(field_type.to_string()),
        ClickhouseColumnType::Boolean => Ok(field_type.to_string()),
        ClickhouseColumnType::ClickhouseInt(int) => match int {
            ClickhouseInt::Int8 => Ok(int.to_string()),
            ClickhouseInt::Int16 => Ok(int.to_string()),
            ClickhouseInt::Int32 => Ok(int.to_string()),
            ClickhouseInt::Int64 => Ok(int.to_string()),
            ClickhouseInt::Int128 => Ok(int.to_string()),
            ClickhouseInt::Int256 => Ok(int.to_string()),
            ClickhouseInt::UInt8 => Ok(int.to_string()),
            ClickhouseInt::UInt16 => Ok(int.to_string()),
            ClickhouseInt::UInt32 => Ok(int.to_string()),
            ClickhouseInt::UInt64 => Ok(int.to_string()),
            ClickhouseInt::UInt128 => Ok(int.to_string()),
            ClickhouseInt::UInt256 => Ok(int.to_string()),
        },
        ClickhouseColumnType::ClickhouseFloat(float) => match float {
            ClickhouseFloat::Float32 => Ok(float.to_string()),
            ClickhouseFloat::Float64 => Ok(float.to_string()),
        },
        ClickhouseColumnType::Decimal => Ok(field_type.to_string()),
        ClickhouseColumnType::DateTime => Ok(field_type.to_string()),
        _ => Err(UnsupportedDataTypeError {
            type_name: field_type.to_string(),
        }),
    }
}

fn clickhouse_column_to_create_table_field_context(
    column: ClickhouseColumn,
) -> Result<CreateTableFieldContext, UnsupportedDataTypeError> {
    if column.arity == FieldArity::List {
        Ok(CreateTableFieldContext {
            field_name: column.name,
            field_type: format!("Array({})", field_type_to_string(column.column_type)?),
            field_arity: if column.arity.is_required() {
                "NOT NULL".to_string()
            } else {
                "NULL".to_string()
            },
        })
    } else {
        Ok(CreateTableFieldContext {
            field_name: column.name,
            field_type: field_type_to_string(column.column_type)?,
            field_arity: if column.arity.is_required() {
                "NOT NULL".to_string()
            } else {
                "NULL".to_string()
            },
        })
    }
}
