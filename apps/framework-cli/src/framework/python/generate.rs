use crate::framework::core::infrastructure::table::{ColumnType, FloatType, IntType, Table};
use convert_case::{Case, Casing};
use std::fmt::Write;

fn map_column_type_to_python(column_type: &ColumnType) -> String {
    match column_type {
        ColumnType::String => "str".to_string(),
        ColumnType::Boolean => "bool".to_string(),
        ColumnType::Int(int_type) => match int_type {
            IntType::Int8 => "Annotated[int, \"int8\"]".to_string(),
            IntType::Int16 => "Annotated[int, \"int16\"]".to_string(),
            IntType::Int32 => "Annotated[int, \"int32\"]".to_string(),
            IntType::Int64 => "Annotated[int, \"int64\"]".to_string(),
            IntType::Int128 => "Annotated[int, \"int128\"]".to_string(),
            IntType::Int256 => "Annotated[int, \"int256\"]".to_string(),
            IntType::UInt8 => "Annotated[int, \"uint8\"]".to_string(),
            IntType::UInt16 => "Annotated[int, \"uint16\"]".to_string(),
            IntType::UInt32 => "Annotated[int, \"uint32\"]".to_string(),
            IntType::UInt64 => "Annotated[int, \"uint64\"]".to_string(),
            IntType::UInt128 => "Annotated[int, \"uint128\"]".to_string(),
            IntType::UInt256 => "Annotated[int, \"uint256\"]".to_string(),
        },
        ColumnType::BigInt => "int".to_string(),
        ColumnType::Float(float_type) => match float_type {
            FloatType::Float32 => "float".to_string(),
            FloatType::Float64 => "float".to_string(),
        },
        ColumnType::Decimal { precision, scale } => format!("Decimal({}, {})", precision, scale),
        ColumnType::DateTime { precision: None } => "datetime".to_string(),
        ColumnType::DateTime {
            precision: Some(precision),
        } => format!("datetime({})", precision),
        ColumnType::Date => "date".to_string(),
        ColumnType::Date16 => "date".to_string(),
        ColumnType::Enum(_) => "TODO".to_string(),
        ColumnType::Array {
            element_type,
            element_nullable,
        } => {
            let inner_type = map_column_type_to_python(element_type);
            let inner_type = if *element_nullable {
                format!("Optional[{}]", inner_type)
            } else {
                inner_type
            };
            format!("list[{}]", inner_type)
        }
        ColumnType::Nested(_) => "TODO".to_string(),
        ColumnType::Json => "Any".to_string(),
        ColumnType::Bytes => "bytes".to_string(),
        ColumnType::Uuid => "UUID".to_string(),
    }
}

pub fn tables_to_python(tables: &[Table]) -> String {
    let mut output = String::new();

    // Add imports
    writeln!(output, "from pydantic import BaseModel").unwrap();
    writeln!(output, "from typing import Optional, Any, Annotated").unwrap();
    writeln!(output, "from datetime import datetime").unwrap();
    writeln!(
        output,
        "from moose_lib import Key, IngestPipeline, IngestPipelineConfig"
    )
    .unwrap();
    writeln!(output).unwrap();

    // Generate model classes
    for table in tables {
        writeln!(output, "class {}(BaseModel):", table.name).unwrap();

        for column in &table.columns {
            let type_str = map_column_type_to_python(&column.data_type);

            let (type_str, default) = if !column.required {
                (format!("Optional[{}]", type_str), " = None")
            } else {
                (type_str, "")
            };

            let type_str = if column.primary_key {
                format!("Key[{}]", type_str)
            } else {
                type_str
            };
            writeln!(output, "    {}: {}{}", column.name, type_str, default).unwrap();
        }
        writeln!(output).unwrap();
    }

    // Generate pipeline configurations
    for table in tables {
        writeln!(
            output,
            "{}_model = IngestPipeline[{}](\"{}\", IngestPipelineConfig(",
            table.name.to_case(Case::Snake),
            table.name,
            table.name
        )
        .unwrap();
        writeln!(output, "    ingest=True,").unwrap();
        writeln!(output, "    stream=True,").unwrap();
        writeln!(output, "    table=True").unwrap();
        writeln!(output, "))").unwrap();
        writeln!(output).unwrap();
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::core::infrastructure::table::{
        Column, ColumnType, DataEnum, EnumMember, EnumValue,
    };
    use crate::framework::core::infrastructure_map::{PrimitiveSignature, PrimitiveTypes};

    #[test]
    fn test_tables_to_python() {
        let tables = vec![Table {
            name: "Foo".to_string(),
            columns: vec![
                Column {
                    name: "primary_key".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: true,
                    default: None,
                    annotations: vec![],
                },
                Column {
                    name: "timestamp".to_string(),
                    data_type: ColumnType::Float(FloatType::Float64),
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                    annotations: vec![],
                },
                Column {
                    name: "optional_text".to_string(),
                    data_type: ColumnType::String,
                    required: false,
                    unique: false,
                    primary_key: false,
                    default: None,
                    annotations: vec![],
                },
            ],
            order_by: vec!["primary_key".to_string()],
            deduplicate: false,
            engine: Some("MergeTree".to_string()),
            version: None,
            source_primitive: PrimitiveSignature {
                name: "Foo".to_string(),
                primitive_type: PrimitiveTypes::DataModel,
            },
            metadata: None,
        }];

        let result = tables_to_python(&tables);

        println!("{}", result);
        assert!(result.contains(
            r#"from pydantic import BaseModel
from typing import Optional, Any, Annotated
from datetime import datetime
from moose_lib import Key, IngestPipeline, IngestPipelineConfig

class Foo(BaseModel):
    primary_key: Key[str]
    timestamp: float
    optional_text: Optional[str] = None

foo_model = IngestPipeline[Foo]("Foo", IngestPipelineConfig(
    ingest=True,
    stream=True,
    table=True
))"#
        ));
    }

    #[test]
    fn test_nested_array_types() {
        let tables = vec![Table {
            name: "NestedArray".to_string(),
            columns: vec![
                Column {
                    name: "id".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: true,
                    default: None,
                    annotations: vec![],
                },
                Column {
                    name: "numbers".to_string(),
                    data_type: ColumnType::Array {
                        element_type: Box::new(ColumnType::Int(IntType::Int32)),
                        element_nullable: false,
                    },
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                    annotations: vec![],
                },
                Column {
                    name: "nested_numbers".to_string(),
                    data_type: ColumnType::Array {
                        element_type: Box::new(ColumnType::Array {
                            element_type: Box::new(ColumnType::Int(IntType::Int32)),
                            element_nullable: true,
                        }),
                        element_nullable: false,
                    },
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                    annotations: vec![],
                },
            ],
            order_by: vec!["id".to_string()],
            deduplicate: false,
            engine: Some("MergeTree".to_string()),
            version: None,
            source_primitive: PrimitiveSignature {
                name: "NestedArray".to_string(),
                primitive_type: PrimitiveTypes::DataModel,
            },
            metadata: None,
        }];

        let result = tables_to_python(&tables);
        println!("{}", result);
        assert!(result.contains(
            r#"class NestedArray(BaseModel):
    id: Key[str]
    numbers: list[Annotated[int, "int32"]]
    nested_numbers: list[list[Optional[Annotated[int, "int32"]]]]

nested_array_model = IngestPipeline[NestedArray]("NestedArray", IngestPipelineConfig(
    ingest=True,
    stream=True,
    table=True
))"#
        ));
    }
}
