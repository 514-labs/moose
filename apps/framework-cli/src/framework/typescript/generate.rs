use crate::framework::core::infrastructure::table::{
    ColumnType, DataEnum, EnumValue, FloatType, Nested, Table,
};
use convert_case::{Case, Casing};
use std::collections::HashMap;
use std::fmt::Write;

fn map_column_type_to_typescript(
    column_type: &ColumnType,
    enums: &HashMap<&DataEnum, String>,
    nested: &HashMap<&Nested, String>,
) -> String {
    match column_type {
        ColumnType::String => "string".to_string(),
        ColumnType::Boolean => "boolean".to_string(),
        ColumnType::Int(int_type) => {
            let lowercase_int_type = format!("{:?}", int_type).to_lowercase();
            format!("number & ClickHouseInt<\"{}\">", lowercase_int_type)
        }
        ColumnType::BigInt => "bigint".to_string(),
        ColumnType::Float(FloatType::Float64) => "number".to_string(),
        ColumnType::Float(FloatType::Float32) => "number & typia.tags.Type<\"float\">".to_string(),
        ColumnType::Decimal { .. } => "number".to_string(),
        ColumnType::DateTime { .. } => "Date".to_string(),
        ColumnType::Date | ColumnType::Date16 => "Date".to_string(),
        ColumnType::Enum(data_enum) => enums.get(data_enum).unwrap().to_string(),
        ColumnType::Array {
            element_type,
            element_nullable,
        } => {
            let inner_type = map_column_type_to_typescript(element_type, enums, nested);
            let inner_type = if *element_nullable {
                format!("({} | null)", inner_type)
            } else {
                inner_type
            };
            format!("{}[]", inner_type)
        }
        ColumnType::Nested(nested_type) => nested.get(nested_type).unwrap().to_string(),
        ColumnType::Json => "Record<string, any>".to_string(),
        ColumnType::Bytes => "Uint8Array".to_string(),
        ColumnType::Uuid => "string & typia.tags.Format<\"uuid\">".to_string(),
    }
}

fn generate_enum(data_enum: &DataEnum, name: &str) -> String {
    let mut enum_def = String::new();
    writeln!(enum_def, "export enum {} {{", name).unwrap();
    for member in &data_enum.values {
        match &member.value {
            EnumValue::Int(i) => writeln!(enum_def, "    {} = {},", member.name, i).unwrap(),
            EnumValue::String(s) => writeln!(enum_def, "    {} = \"{}\",", member.name, s).unwrap(),
        }
    }
    writeln!(enum_def, "}}").unwrap();
    writeln!(enum_def).unwrap();
    enum_def
}

fn generate_interface(
    nested: &Nested,
    name: &str,
    enums: &HashMap<&DataEnum, String>,
    nested_models: &HashMap<&Nested, String>,
) -> String {
    let mut interface = String::new();
    writeln!(interface, "export interface {} {{", name).unwrap();

    for column in &nested.columns {
        let type_str = map_column_type_to_typescript(&column.data_type, enums, nested_models);
        let type_str = if column.primary_key {
            format!("Key<{}>", type_str)
        } else {
            type_str
        };
        let type_str = if !column.required {
            format!("{} | undefined", type_str)
        } else {
            type_str
        };
        writeln!(
            interface,
            "    {}: {};",
            column.name.to_case(Case::Camel),
            type_str
        )
        .unwrap();
    }
    writeln!(interface, "}}").unwrap();
    writeln!(interface).unwrap();
    interface
}

pub fn tables_to_typescript(tables: &[Table]) -> String {
    let mut output = String::new();

    // Add imports
    writeln!(
        output,
        "import {{ IngestPipeline, Key, ClickHouseInt }} from \"@514labs/moose-lib\";"
    )
    .unwrap();
    writeln!(output, "import typia from \"typia\";").unwrap();
    writeln!(output).unwrap();

    // Collect all enums and nested types
    let mut enums: HashMap<&DataEnum, String> = HashMap::new();
    let mut nested_models: HashMap<&Nested, String> = HashMap::new();

    // First pass: collect all nested types and enums
    for table in tables {
        for column in &table.columns {
            match &column.data_type {
                ColumnType::Enum(data_enum) => {
                    enums.insert(data_enum, column.name.to_case(Case::Pascal));
                }
                ColumnType::Nested(nested) => {
                    nested_models.insert(nested, column.name.to_case(Case::Pascal));
                }
                _ => {}
            }
        }
    }

    // Generate enum definitions
    for (data_enum, name) in enums.iter() {
        output.push_str(&generate_enum(data_enum, name));
    }

    // Generate nested interface definitions
    for (nested, name) in nested_models.iter() {
        output.push_str(&generate_interface(nested, name, &enums, &nested_models));
    }

    // Generate model interfaces
    for table in tables {
        writeln!(output, "export interface {} {{", table.name).unwrap();

        for column in &table.columns {
            let type_str = map_column_type_to_typescript(&column.data_type, &enums, &nested_models);
            let type_str = if column.primary_key {
                format!("Key<{}>", type_str)
            } else {
                type_str
            };
            let type_str = if !column.required {
                format!("{} | undefined", type_str)
            } else {
                type_str
            };
            writeln!(
                output,
                "    {}: {};",
                column.name.to_case(Case::Camel),
                type_str
            )
            .unwrap();
        }
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();
    }

    // Generate pipeline configurations
    for table in tables {
        writeln!(
            output,
            "export const {}Pipeline = new IngestPipeline<{}>(\"{}\", {{",
            table.name.to_case(Case::Pascal),
            table.name,
            table.name
        )
        .unwrap();
        writeln!(output, "    table: {},", table.engine.is_some()).unwrap();
        writeln!(output, "    stream: true,").unwrap();
        writeln!(output, "    ingest: true,").unwrap();
        writeln!(output, "}});").unwrap();
        writeln!(output).unwrap();
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::core::infrastructure::table::{Column, ColumnType, EnumMember, Nested};
    use crate::framework::core::infrastructure_map::{PrimitiveSignature, PrimitiveTypes};

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

        let result = tables_to_typescript(&tables);
        println!("{}", result);
        assert!(result.contains(
            r#"import { IngestPipeline, Key, ClickHouseInt } from "@514labs/moose-lib";
import typia from "typia";

/**
 * Data Pipeline: Raw Record → Processed Record
 * Raw → HTTP → Raw Stream → Transform → Derived → Processed Stream → DB Table
 */

/** =======Data Models========= */

/** Nested type for Address
 * @see https://docs.moosejs.com/nested-types
 */
export interface Address {
    street: string;
    city: string;
    zipCode?: string;
}

/** Data model for User */
export interface User {
    id: Key<string>;
    address: Address;
    addresses?: Address[];
}

/** =======Pipeline Configuration========= */

/** Pipeline configuration for User
 * @see https://docs.moosejs.com/pipelines
 */
export const UserPipeline = new IngestPipeline<User>("User", {
    table: true,
    stream: true,
    ingest: true,
});"#
        ));
    }

    #[test]
    fn test_enum_types() {
        let status_enum = DataEnum {
            name: "Status".to_string(),
            values: vec![
                EnumMember {
                    name: "OK".to_string(),
                    value: EnumValue::String("ok".to_string()),
                },
                EnumMember {
                    name: "ERROR".to_string(),
                    value: EnumValue::String("error".to_string()),
                },
            ],
        };

        let tables = vec![Table {
            name: "Task".to_string(),
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
                    name: "status".to_string(),
                    data_type: ColumnType::Enum(status_enum),
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
                name: "Task".to_string(),
                primitive_type: PrimitiveTypes::DataModel,
            },
            metadata: None,
        }];

        let result = tables_to_typescript(&tables);
        println!("{}", result);
        assert!(result.contains(
            r#"import { IngestPipeline, Key, ClickHouseInt } from "@514labs/moose-lib";
import typia from "typia";

/**
 * Data Pipeline: Raw Record → Processed Record
 * Raw → HTTP → Raw Stream → Transform → Derived → Processed Stream → DB Table
 */

/** =======Data Models========= */

/** Enum values for Status
 * @see https://docs.moosejs.com/enums
 */
export enum Status {
    OK = "ok",
    ERROR = "error",
}

/** Data model for Task */
export interface Task {
    id: Key<string>;
    status: Status;
}

/** =======Pipeline Configuration========= */

/** Pipeline configuration for Task
 * @see https://docs.moosejs.com/pipelines
 */
export const TaskPipeline = new IngestPipeline<Task>("Task", {
    table: true,
    stream: true,
    ingest: true,
});"#
        ));
    }
}
