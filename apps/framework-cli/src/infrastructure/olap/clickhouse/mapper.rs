use crate::framework::schema::{ColumnType, FieldArity, Table, TableType};

use crate::infrastructure::olap::clickhouse::model::{
    ClickHouseColumn, ClickHouseColumnType, ClickHouseFloat, ClickHouseInt, ClickHouseTable,
    ClickHouseTableType,
};

use super::model::sanitize_column_name;

pub fn clickhouse_table_type_mapper(table_type: TableType) -> ClickHouseTableType {
    match table_type {
        TableType::Table => ClickHouseTableType::Table,
        _ => ClickHouseTableType::Unsupported,
    }
}

pub fn std_field_type_to_clickhouse_type_mapper(field_type: ColumnType) -> ClickHouseColumnType {
    match field_type {
        ColumnType::String => ClickHouseColumnType::String,
        ColumnType::Boolean => ClickHouseColumnType::Boolean,
        ColumnType::Int => ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int64),
        ColumnType::Float => ClickHouseColumnType::ClickhouseFloat(ClickHouseFloat::Float64),
        ColumnType::Decimal => ClickHouseColumnType::Decimal,
        ColumnType::DateTime => ClickHouseColumnType::DateTime,
        ColumnType::Enum(x) => ClickHouseColumnType::Enum(x),
        ColumnType::Unsupported => ClickHouseColumnType::Unsupported,
        _ => ClickHouseColumnType::Unsupported,
    }
}

pub fn arity_mapper(arity: schema_ast::ast::FieldArity) -> FieldArity {
    match arity {
        schema_ast::ast::FieldArity::Required => FieldArity::Required,
        schema_ast::ast::FieldArity::Optional => FieldArity::Optional,
        schema_ast::ast::FieldArity::List => FieldArity::List,
    }
}

pub fn std_table_to_clickhouse_table(table: Table) -> ClickHouseTable {
    let columns = table
        .columns
        .into_iter()
        .map(|column| {
            ClickHouseColumn {
                name: sanitize_column_name(column.name),
                column_type: std_field_type_to_clickhouse_type_mapper(column.data_type),
                arity: column.arity,
                unique: column.unique,
                primary_key: column.primary_key,
                default: None, // TODO: Implement the default mapper
            }
        })
        .collect::<Vec<ClickHouseColumn>>();

    ClickHouseTable {
        db_name: table.db_name,
        name: table.name,
        columns,
        table_type: clickhouse_table_type_mapper(table.table_type),
    }
}
