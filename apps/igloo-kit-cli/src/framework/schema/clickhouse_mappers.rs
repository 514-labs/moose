use crate::infrastructure::db::clickhouse::{ClickhouseColumnType, ClickhouseInt, ClickhouseFloat, ClickhouseTableType, ClickhouseTable, ClickhouseColumn};

use super::{ColumnType, TableType, Table};

pub fn clickhouse_table_type_mapper(table_type: TableType) -> ClickhouseTableType {
    match table_type {
        TableType::Table => ClickhouseTableType::Table,
        _ => ClickhouseTableType::Unsupported,
    }
}

pub fn std_field_type_to_clickhouse_type_mapper(field_type: ColumnType) -> ClickhouseColumnType {
    match field_type {
        ColumnType::String => ClickhouseColumnType::String,
        ColumnType::Boolean => ClickhouseColumnType::Boolean,
        ColumnType::Int => ClickhouseColumnType::ClickhouseInt(ClickhouseInt::Int64),
        ColumnType::Float => ClickhouseColumnType::ClickhouseFloat(ClickhouseFloat::Float64),
        ColumnType::Decimal => ClickhouseColumnType::Decimal,
        ColumnType::DateTime => ClickhouseColumnType::DateTime,
        ColumnType::Unsupported => ClickhouseColumnType::Unsupported,
        _ => ClickhouseColumnType::Unsupported,
    }
}

pub fn std_table_to_clickhouse_table (db_name: String, table: Table) -> ClickhouseTable {
    let columns = table.columns.into_iter().map(|column| {
        ClickhouseColumn {
            name: column.name,
            column_type: std_field_type_to_clickhouse_type_mapper(column.data_type),
            arity: column.arity,
            unique: column.unique,
            primary_key: column.primary_key,
            default: None, // TODO: Implement the default mapper
        }
    }).collect::<Vec<ClickhouseColumn>>();

    ClickhouseTable {
        db_name,
        name: table.name,
        columns,
        table_type: clickhouse_table_type_mapper(table.table_type),
    }
}