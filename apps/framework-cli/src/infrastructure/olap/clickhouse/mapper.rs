use crate::{
    framework::schema::{ColumnType, FieldArity, Table, TableType},
    infrastructure::olap::clickhouse::{
        ClickhouseColumn, ClickhouseColumnType, ClickhouseFloat, ClickhouseInt, ClickhouseTable,
        ClickhouseTableType,
    },
};

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
        ColumnType::Enum(x) => ClickhouseColumnType::Enum(x),
        ColumnType::Unsupported => ClickhouseColumnType::Unsupported,
        _ => ClickhouseColumnType::Unsupported,
    }
}

pub fn arity_mapper(arity: schema_ast::ast::FieldArity) -> FieldArity {
    match arity {
        schema_ast::ast::FieldArity::Required => FieldArity::Required,
        schema_ast::ast::FieldArity::Optional => FieldArity::Optional,
        schema_ast::ast::FieldArity::List => FieldArity::List,
    }
}

pub fn std_table_to_clickhouse_table(table: Table) -> ClickhouseTable {
    let columns = table
        .columns
        .into_iter()
        .map(|column| {
            ClickhouseColumn {
                name: column.name,
                column_type: std_field_type_to_clickhouse_type_mapper(column.data_type),
                arity: column.arity,
                unique: column.unique,
                primary_key: column.primary_key,
                default: None, // TODO: Implement the default mapper
            }
        })
        .collect::<Vec<ClickhouseColumn>>();

    ClickhouseTable {
        db_name: table.db_name,
        name: table.name,
        columns,
        table_type: clickhouse_table_type_mapper(table.table_type),
    }
}
