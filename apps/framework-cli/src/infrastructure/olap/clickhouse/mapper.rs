use crate::framework::core::infrastructure::table::{
    Column, ColumnType, FloatType, IntType, Table,
};
use serde_json::Value;

use crate::infrastructure::olap::clickhouse::model::{
    AggregationFunction, ClickHouseColumn, ClickHouseColumnType, ClickHouseFloat, ClickHouseInt,
    ClickHouseTable,
};

use super::errors::ClickhouseError;
use super::model::sanitize_column_name;
use super::queries::ClickhouseEngine;

pub fn std_column_to_clickhouse_column(
    column: Column,
) -> Result<ClickHouseColumn, ClickhouseError> {
    let clickhouse_column = ClickHouseColumn {
        name: sanitize_column_name(column.name),
        column_type: std_field_type_to_clickhouse_type_mapper(
            column.data_type,
            &column.annotations,
        )?,
        required: column.required,
        unique: column.unique,
        primary_key: column.primary_key,
        default: None, // TODO: Implement the default mapper
    };

    Ok(clickhouse_column)
}

pub fn std_field_type_to_clickhouse_type_mapper(
    field_type: ColumnType,
    annotations: &[(String, Value)],
) -> Result<ClickHouseColumnType, ClickhouseError> {
    if let Some((_, agg_func)) = annotations.iter().find(|(k, _)| k == "aggregationFunction") {
        let clickhouse_type = std_field_type_to_clickhouse_type_mapper(field_type, &[])?;

        let agg_func =
            serde_json::from_value::<AggregationFunction<ColumnType>>(agg_func.clone()).unwrap();

        return Ok(ClickHouseColumnType::AggregateFunction(
            AggregationFunction {
                function_name: agg_func.function_name,
                argument_types: agg_func
                    .argument_types
                    .into_iter()
                    .map(|t| std_field_type_to_clickhouse_type_mapper(t, &[]))
                    .collect::<Result<Vec<_>, _>>()?,
            },
            Box::new(clickhouse_type),
        ));
    }

    if annotations
        .iter()
        .any(|(k, v)| k == "LowCardinality" && v == &serde_json::json!(true))
    {
        let clickhouse_type = std_field_type_to_clickhouse_type_mapper(field_type, &[])?;
        return Ok(ClickHouseColumnType::LowCardinality(Box::new(
            clickhouse_type,
        )));
    }

    match field_type {
        ColumnType::String => Ok(ClickHouseColumnType::String),
        ColumnType::Boolean => Ok(ClickHouseColumnType::Boolean),
        ColumnType::Int(IntType::Int8) => {
            Ok(ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int8))
        }
        ColumnType::Int(IntType::Int16) => {
            Ok(ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int16))
        }
        ColumnType::Int(IntType::Int32) => {
            Ok(ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int32))
        }
        ColumnType::Int(IntType::Int64) => {
            Ok(ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int64))
        }
        ColumnType::Int(IntType::Int128) => {
            Ok(ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int128))
        }
        ColumnType::Int(IntType::Int256) => {
            Ok(ClickHouseColumnType::ClickhouseInt(ClickHouseInt::Int256))
        }
        ColumnType::Int(IntType::UInt8) => {
            Ok(ClickHouseColumnType::ClickhouseInt(ClickHouseInt::UInt8))
        }
        ColumnType::Int(IntType::UInt16) => {
            Ok(ClickHouseColumnType::ClickhouseInt(ClickHouseInt::UInt16))
        }
        ColumnType::Int(IntType::UInt32) => {
            Ok(ClickHouseColumnType::ClickhouseInt(ClickHouseInt::UInt32))
        }
        ColumnType::Int(IntType::UInt64) => {
            Ok(ClickHouseColumnType::ClickhouseInt(ClickHouseInt::UInt64))
        }
        ColumnType::Int(IntType::UInt128) => {
            Ok(ClickHouseColumnType::ClickhouseInt(ClickHouseInt::UInt128))
        }
        ColumnType::Int(IntType::UInt256) => {
            Ok(ClickHouseColumnType::ClickhouseInt(ClickHouseInt::UInt256))
        }
        ColumnType::Float(FloatType::Float32) => Ok(ClickHouseColumnType::ClickhouseFloat(
            ClickHouseFloat::Float32,
        )),
        ColumnType::Float(FloatType::Float64) => Ok(ClickHouseColumnType::ClickhouseFloat(
            ClickHouseFloat::Float64,
        )),
        ColumnType::Decimal { precision, scale } => {
            Ok(ClickHouseColumnType::Decimal { precision, scale })
        }
        ColumnType::DateTime { precision: None } => Ok(ClickHouseColumnType::DateTime),
        ColumnType::DateTime {
            precision: Some(precision),
        } => Ok(ClickHouseColumnType::DateTime64 { precision }),
        ColumnType::Enum(x) => Ok(ClickHouseColumnType::Enum(x)),
        ColumnType::Array {
            element_type,
            element_nullable,
        } => {
            let inner_clickhouse_type =
                std_field_type_to_clickhouse_type_mapper(*element_type, &[])?;
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
        ColumnType::Uuid => Ok(ClickHouseColumnType::Uuid),
        ColumnType::Date => Ok(ClickHouseColumnType::Date32),
        ColumnType::Date16 => Ok(ClickHouseColumnType::Date),
        ColumnType::IpV4 => Ok(ClickHouseColumnType::IpV4),
        ColumnType::IpV6 => Ok(ClickHouseColumnType::IpV6),
        ColumnType::Nullable(inner) => {
            let inner_type = std_field_type_to_clickhouse_type_mapper(*inner, &[])?;
            Ok(ClickHouseColumnType::Nullable(Box::new(inner_type)))
        }
        ColumnType::NamedTuple(fields) => Ok(ClickHouseColumnType::NamedTuple(
            fields
                .into_iter()
                .map(|(name, t)| {
                    Ok::<_, ClickhouseError>((
                        name,
                        std_field_type_to_clickhouse_type_mapper(t, &[])?,
                    ))
                })
                .collect::<Result<_, _>>()?,
        )),
        ColumnType::Map {
            key_type,
            value_type,
        } => {
            let clickhouse_key_type = std_field_type_to_clickhouse_type_mapper(*key_type, &[])?;
            let clickhouse_value_type = std_field_type_to_clickhouse_type_mapper(*value_type, &[])?;
            Ok(ClickHouseColumnType::Map(
                Box::new(clickhouse_key_type),
                Box::new(clickhouse_value_type),
            ))
        }
    }
}

pub fn std_columns_to_clickhouse_columns(
    columns: &Vec<Column>,
) -> Result<Vec<ClickHouseColumn>, ClickhouseError> {
    let mut clickhouse_columns: Vec<ClickHouseColumn> = Vec::new();
    for column in columns {
        let clickhouse_column = ClickHouseColumn {
            name: sanitize_column_name(column.name.clone()),
            column_type: std_field_type_to_clickhouse_type_mapper(
                column.data_type.clone(),
                &column.annotations,
            )?,
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

    let clickhouse_engine = match &table.engine {
        Some(engine) => ClickhouseEngine::try_from(engine.as_str()).map_err(|e| {
            ClickhouseError::InvalidParameters {
                message: format!("engine: {e}"),
            }
        })?,
        None if table.deduplicate => ClickhouseEngine::ReplacingMergeTree,
        None => ClickhouseEngine::MergeTree,
    };

    Ok(ClickHouseTable {
        name: table.name.clone(),
        version: table.version.clone(),
        columns,
        order_by: table.order_by.clone(),
        engine: clickhouse_engine,
    })
}
