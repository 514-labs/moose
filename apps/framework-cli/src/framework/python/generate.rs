use crate::framework::core::infrastructure::table::{
    ColumnType, DataEnum, EnumValue, FloatType, IntType, Nested, Table,
};
use convert_case::{Case, Casing};
use regex::Regex;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::LazyLock;

fn map_column_type_to_python(
    column_type: &ColumnType,
    enums: &HashMap<&DataEnum, String>,
    nested: &HashMap<&Nested, String>,
) -> String {
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
            FloatType::Float32 => "Annotated[float, ClickhouseSize(4)]".to_string(),
            FloatType::Float64 => "float".to_string(),
        },
        ColumnType::Decimal { precision, scale } => {
            format!("clickhouse_decimal({}, {})", precision, scale)
        }
        ColumnType::DateTime { precision: None } => "datetime.datetime".to_string(),
        ColumnType::DateTime {
            precision: Some(precision),
        } => format!("clickhouse_datetime64({})", precision),
        ColumnType::Date => "datetime.date".to_string(),
        ColumnType::Date16 => "Annotated[datetime.date, ClickhouseSize(2)]".to_string(),
        ColumnType::Enum(data_enum) => enums.get(data_enum).unwrap().to_string(),
        ColumnType::Array {
            element_type,
            element_nullable,
        } => {
            let inner_type = map_column_type_to_python(element_type, enums, nested);
            let inner_type = if *element_nullable {
                format!("Optional[{}]", inner_type)
            } else {
                inner_type
            };
            format!("list[{}]", inner_type)
        }
        ColumnType::Nested(nested_type) => nested.get(nested_type).unwrap().to_string(),
        ColumnType::Json => "Any".to_string(),
        ColumnType::Bytes => "bytes".to_string(),
        ColumnType::Uuid => "UUID".to_string(),
        ColumnType::IpV4 => "ipaddress.IPv4Address".to_string(),
        ColumnType::IpV6 => "ipaddress.IPv6Address".to_string(),
    }
}

fn generate_enum_class(data_enum: &DataEnum, name: &str) -> String {
    let mut enum_class = String::new();
    writeln!(
        enum_class,
        "class {}({}):",
        name,
        if data_enum
            .values
            .iter()
            .all(|v| matches!(v.value, EnumValue::Int(_)))
        {
            "StringToEnumMixin, IntEnum"
        } else {
            "Enum"
        }
    )
    .unwrap();
    for member in &data_enum.values {
        match &member.value {
            EnumValue::Int(i) => {
                if PYTHON_IDENTIFIER_PATTERN.is_match(&member.name) {
                    writeln!(enum_class, "    {} = {}", member.name, i).unwrap();
                } else {
                    // skip names that are not valid identifiers
                    writeln!(enum_class, "    # {} = \"{}\"", member.name, i).unwrap()
                }
            }
            EnumValue::String(s) => {
                writeln!(enum_class, "    {} = \"{}\"", member.name, s).unwrap()
            }
        }
    }
    writeln!(enum_class).unwrap();
    enum_class
}

const PYTHON_IDENTIFIER_REGEX: &str = r"^[^\d\W]\w*$";
pub static PYTHON_IDENTIFIER_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(PYTHON_IDENTIFIER_REGEX).unwrap());

fn generate_nested_model(
    nested: &Nested,
    name: &str,
    enums: &HashMap<&DataEnum, String>,
    nested_models: &HashMap<&Nested, String>,
) -> String {
    let mut model = String::new();
    writeln!(model, "class {}(BaseModel):", name).unwrap();

    for column in &nested.columns {
        let type_str = map_column_type_to_python(&column.data_type, enums, nested_models);

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
        writeln!(model, "    {}: {}{}", column.name, type_str, default).unwrap();
    }
    writeln!(model).unwrap();
    model
}

pub fn tables_to_python(tables: &[Table]) -> String {
    let mut output = String::new();

    // Add imports
    writeln!(output, "from pydantic import BaseModel").unwrap();
    writeln!(output, "from typing import Optional, Any, Annotated").unwrap();
    writeln!(output, "import datetime").unwrap();
    writeln!(output, "import ipaddress").unwrap();
    writeln!(output, "from uuid import UUID").unwrap();
    writeln!(output, "from enum import IntEnum, Enum").unwrap();
    writeln!(
        output,
        "from moose_lib import Key, IngestPipeline, IngestPipelineConfig, clickhouse_datetime64, clickhouse_decimal, ClickhouseSize, StringToEnumMixin"
    )
    .unwrap();
    writeln!(output).unwrap();

    // Collect all enums and nested types
    let mut enums: HashMap<&DataEnum, String> = HashMap::new();
    let mut extra_class_names: HashMap<String, usize> = HashMap::new();
    let mut nested_models: HashMap<&Nested, String> = HashMap::new();

    // First pass: collect all nested types and enums
    for table in tables {
        for column in &table.columns {
            match &column.data_type {
                ColumnType::Enum(data_enum) => {
                    if !enums.contains_key(data_enum) {
                        let name = column.name.to_case(Case::Pascal);
                        let name = match extra_class_names.entry(name.clone()) {
                            Entry::Occupied(mut entry) => {
                                *entry.get_mut() = entry.get() + 1;
                                format!("{}{}", name, entry.get())
                            }
                            Entry::Vacant(entry) => {
                                entry.insert(0);
                                name
                            }
                        };
                        enums.insert(data_enum, name);
                    }
                }
                ColumnType::Nested(nested) => {
                    if !nested_models.contains_key(nested) {
                        let name = column.name.to_case(Case::Pascal);
                        let name = match extra_class_names.entry(name.clone()) {
                            Entry::Occupied(mut entry) => {
                                *entry.get_mut() = entry.get() + 1;
                                format!("{}{}", name, entry.get())
                            }
                            Entry::Vacant(entry) => {
                                entry.insert(0);
                                name
                            }
                        };
                        nested_models.insert(nested, name);
                    }
                }
                _ => {}
            }
        }
    }

    // Generate enum classes
    for (data_enum, name) in enums.iter() {
        output.push_str(&generate_enum_class(data_enum, name));
    }

    // Generate nested model classes
    for (nested, name) in nested_models.iter() {
        output.push_str(&generate_nested_model(nested, name, &enums, &nested_models));
    }

    // Generate model classes
    for table in tables {
        writeln!(output, "class {}(BaseModel):", table.name).unwrap();

        for column in &table.columns {
            let type_str = map_column_type_to_python(&column.data_type, &enums, &nested_models);

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
    use crate::framework::core::infrastructure::table::{Column, ColumnType, Nested};
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
import datetime
import ipaddress
from uuid import UUID
from enum import IntEnum, Enum
from moose_lib import Key, IngestPipeline, IngestPipelineConfig, clickhouse_datetime64, clickhouse_decimal, ClickhouseSize, StringToEnumMixin

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

    #[test]
    fn test_nested_types() {
        let address_nested = Nested {
            name: "Address".to_string(),
            columns: vec![
                Column {
                    name: "street".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                    annotations: vec![],
                },
                Column {
                    name: "city".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                    annotations: vec![],
                },
                Column {
                    name: "zip_code".to_string(),
                    data_type: ColumnType::String,
                    required: false,
                    unique: false,
                    primary_key: false,
                    default: None,
                    annotations: vec![],
                },
            ],
            jwt: false,
        };

        let tables = vec![Table {
            name: "User".to_string(),
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
                    name: "address".to_string(),
                    data_type: ColumnType::Nested(address_nested.clone()),
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                    annotations: vec![],
                },
                Column {
                    name: "addresses".to_string(),
                    data_type: ColumnType::Array {
                        element_type: Box::new(ColumnType::Nested(address_nested)),
                        element_nullable: false,
                    },
                    required: false,
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
                name: "User".to_string(),
                primitive_type: PrimitiveTypes::DataModel,
            },
            metadata: None,
        }];

        let result = tables_to_python(&tables);
        println!("{}", result);
        assert!(result.contains(
            r#"class Address(BaseModel):
    street: str
    city: str
    zip_code: Optional[str] = None

class User(BaseModel):
    id: Key[str]
    address: Address
    addresses: Optional[list[Address]] = None

user_model = IngestPipeline[User]("User", IngestPipelineConfig(
    ingest=True,
    stream=True,
    table=True
))"#
        ));
    }
}
