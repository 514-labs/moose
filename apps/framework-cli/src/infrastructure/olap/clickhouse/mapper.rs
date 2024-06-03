use crate::framework::data_model::schema::{Column, ColumnType, Table, TableType};

use crate::infrastructure::olap::clickhouse::model::{
    ClickHouseColumn, ClickHouseColumnType, ClickHouseFloat, ClickHouseInt, ClickHouseTable,
    ClickHouseTableType,
};

use super::errors::ClickhouseError;
use super::model::sanitize_column_name;

pub fn clickhouse_table_type_mapper(table_type: TableType) -> ClickHouseTableType {
    match table_type {
        TableType::Table => ClickHouseTableType::Table,
        _ => ClickHouseTableType::Unsupported,
    }
}

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
        ColumnType::Array(inner_std_type) => {
            let inner_clickhouse_type = std_field_type_to_clickhouse_type_mapper(*inner_std_type)?;
            Ok(ClickHouseColumnType::Array(Box::new(inner_clickhouse_type)))
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
        ColumnType::Json => Err(ClickhouseError::UnsupportedDataType {
            type_name: "Json".to_string(),
        }),
        ColumnType::Bytes => Err(ClickhouseError::UnsupportedDataType {
            type_name: "Bytes".to_string(),
        }),
    }
}

pub fn std_table_to_clickhouse_table(table: Table) -> Result<ClickHouseTable, ClickhouseError> {
    let mut columns = Vec::new();
    for column in table.columns {
        let clickhouse_column = ClickHouseColumn {
            name: sanitize_column_name(column.name),
            column_type: std_field_type_to_clickhouse_type_mapper(column.data_type)?,
            required: column.required,
            unique: column.unique,
            primary_key: column.primary_key,
            default: None, // TODO: Implement the default mapper
        };
        columns.push(clickhouse_column);
    }

    Ok(ClickHouseTable {
        name: table.name,
        columns,
        table_type: clickhouse_table_type_mapper(table.table_type),
        order_by: table.order_by,
    })
}
