use crate::framework::core::infrastructure::table::{Column, ColumnType, Table};

use crate::infrastructure::olap::clickhouse::model::{
    ClickHouseColumn, ClickHouseColumnType, ClickHouseFloat, ClickHouseInt, ClickHouseTable,
};

use super::errors::ClickhouseError;
use super::model::sanitize_column_name;
use super::queries::ClickhouseEngine;

pub fn std_column_to_clickhouse_column(
    column: Column,
) -> Result<ClickHouseColumn, ClickhouseError> {
    let clickhouse_column = ClickHouseColumn {
        name: sanitize_column_name(column.name),
        column_type: std_field_type_to_clickhouse_type_mapper(column.data_type)?,
        required: column.required,
        unique: column.unique,
        primary_key: column.primary_key,
        default: None, // TODO: Implement the default mapper
    };

    Ok(clickhouse_column)
}

pub fn std_field_type_to_clickhouse_type_mapper(
    field_type: ColumnType,
) -> Result<ClickHouseColumnType, ClickhouseError> {
    match field_type {
        ColumnType::String => Ok(ClickHouseColumnType::String),
        ColumnType::Boolean => Ok(ClickHouseColumnType::Boolean),
        ColumnType::Int => Ok(ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int64)),
        ColumnType::Float => Ok(ClickHouseColumnType::ClickhouseFloat(
            ClickHouseFloat::Float64,
        )),
        ColumnType::Decimal => Ok(ClickHouseColumnType::Decimal),
        ColumnType::DateTime => Ok(ClickHouseColumnType::DateTime),
        ColumnType::Enum(x) => Ok(ClickHouseColumnType::Enum(x)),
        ColumnType::Array {
            element_type,
            element_nullable,
        } => {
            let inner_clickhouse_type = std_field_type_to_clickhouse_type_mapper(*element_type)?;
            let with_nullable = if element_nullable {
                ClickHouseColumnType::Nullable(Box::new(inner_clickhouse_type))
            } else {
                inner_clickhouse_type
            };
            Ok(ClickHouseColumnType::Array(Box::new(with_nullable)))
        }
        ColumnType::Nested(inner_nested) => {
            let column_types = inner_nested
                .columns
                .iter()
                .map(|column| std_column_to_clickhouse_column(column.clone()))
                .collect::<Result<Vec<ClickHouseColumn>, ClickhouseError>>()?;

            Ok(ClickHouseColumnType::Nested(column_types))
        }
        ColumnType::BigInt => Err(ClickhouseError::UnsupportedDataType {
            type_name: "BigInt".to_string(),
        }),
        ColumnType::Json => Ok(ClickHouseColumnType::Json),
        ColumnType::Bytes => Err(ClickhouseError::UnsupportedDataType {
            type_name: "Bytes".to_string(),
        }),
    }
}

pub fn std_columns_to_clickhouse_columns(
    columns: &Vec<Column>,
) -> Result<Vec<ClickHouseColumn>, ClickhouseError> {
    let mut clickhouse_columns: Vec<ClickHouseColumn> = Vec::new();
    for column in columns {
        let clickhouse_column = ClickHouseColumn {
            name: sanitize_column_name(column.name.clone()),
            column_type: std_field_type_to_clickhouse_type_mapper(column.data_type.clone())?,
            required: column.required,
            unique: column.unique,
            primary_key: column.primary_key,
            default: None, // TODO: Implement the default mapper
        };
        clickhouse_columns.push(clickhouse_column);
    }

    Ok(clickhouse_columns)
}

pub fn std_table_to_clickhouse_table(table: &Table) -> Result<ClickHouseTable, ClickhouseError> {
    let columns = std_columns_to_clickhouse_columns(&table.columns)?;

    let clickhouse_engine = if table.deduplicate {
        ClickhouseEngine::ReplacingMergeTree
    } else {
        ClickhouseEngine::MergeTree
    };

    Ok(ClickHouseTable {
        name: table.name.clone(),
        version: table.version.clone(),
        columns,
        order_by: table.order_by.clone(),
        engine: clickhouse_engine,
    })
}
