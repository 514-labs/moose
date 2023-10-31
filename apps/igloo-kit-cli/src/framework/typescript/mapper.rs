use crate::framework::schema::{Table, Column, ColumnType};

use super::{TypescriptInterface, InterfaceField, InterfaceFieldType};

pub fn std_field_type_to_typescript_field_mapper(field_type: ColumnType) -> InterfaceFieldType {
    match field_type {
        ColumnType::String => InterfaceFieldType::String,
        ColumnType::Boolean => InterfaceFieldType::Boolean,
        ColumnType::Int => InterfaceFieldType::Number,
        ColumnType::Float => InterfaceFieldType::Number,
        ColumnType::Decimal => InterfaceFieldType::Number,
        ColumnType::DateTime => InterfaceFieldType::Date,
        ColumnType::Unsupported => InterfaceFieldType::Unsupported,
        _ => InterfaceFieldType::Unsupported,
    }
}


pub fn std_table_to_typescript_interface (table: Table) -> TypescriptInterface {
    let fields = table.columns.into_iter().map(|column: Column| {
        let is_optional = match column.arity {
            schema_ast::ast::FieldArity::Required => false,
            schema_ast::ast::FieldArity::Optional => true,
            schema_ast::ast::FieldArity::List => false,
        };

        InterfaceField {
            name: column.name,
            field_type: std_field_type_to_typescript_field_mapper(column.data_type.clone()),
            is_optional,
            comment: Some(format!("db_type:{} | isPrimary:{}", column.data_type, column.primary_key)),
        }
    }).collect::<Vec<InterfaceField>>();

    TypescriptInterface {
        name: table.name,
        fields
    }
}