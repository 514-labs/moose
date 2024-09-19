use crate::cli::display::{Message, MessageType};
use crate::framework::languages::SupportedLanguages;
use convert_case::{Case, Casing};
use indexmap::IndexMap;
use serde_json::{Deserializer, Value};
use std::collections::HashSet;
use std::io::BufReader;

#[derive(Debug, Clone, PartialEq, Eq)]
enum CustomValue {
    UnionTypes(Vec<CustomValue>),
    // if heterogeneous array (which we don't really support), then the inner value is UnionTypes
    JsonArray(Box<CustomValue>),
    JsonObject(IndexMap<String, CustomValue>),
    JsonPrimitive(JsonPrimitive),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum JsonPrimitive {
    Null,
    String,
    Int,
    Float,
    Bool,
}
pub fn parse_and_generate(name: &str, file: String, language: SupportedLanguages) -> String {
    let map = parse_json_file(&file).unwrap();
    let schema = match language {
        SupportedLanguages::Typescript => render_typescript_interface(&name, &map),
        SupportedLanguages::Python => render_python_dataclass(&name, &map),
    };
    schema
}

fn parse_json_file(file_content: &str) -> Result<IndexMap<String, CustomValue>, serde_json::Error> {
    let reader = BufReader::new(file_content.as_bytes());
    let stream = Deserializer::from_reader(reader).into_iter::<Value>();

    let mut schema = IndexMap::new();

    let mut i = 0;
    for obj in stream {
        if let Value::Array(array) = obj? {
            for obj in array {
                let parsed_map = parse_json(&obj);
                i += 1;
                if i % 100 == 0 {
                    show_message!(
                        MessageType::Info,
                        Message {
                            action: "Init Datamodel".to_string(),
                            details: format!("Processed {:?} records", i),
                        }
                    );
                }
                if i == 1000 {
                    show_message!(
                        MessageType::Info,
                        Message {
                            action: "Init Datamodel".to_string(),
                            details: format!("Processed {:?} records", i),
                        }
                    );
                    break;
                }
                schema = merge_maps(schema, parsed_map);
            }
        }
    }

    Ok(schema)
}

fn parse_json_value(json_data: &Value) -> CustomValue {
    match json_data {
        Value::Null => CustomValue::JsonPrimitive(JsonPrimitive::Null),
        Value::String(_) => CustomValue::JsonPrimitive(JsonPrimitive::String),
        Value::Number(n) if n.is_f64() => CustomValue::JsonPrimitive(JsonPrimitive::Float),
        Value::Number(_integer) => CustomValue::JsonPrimitive(JsonPrimitive::Int),
        Value::Bool(_) => CustomValue::JsonPrimitive(JsonPrimitive::Bool),
        Value::Object(_) => CustomValue::JsonObject(parse_json(json_data)),
        Value::Array(values) => CustomValue::JsonArray(Box::new(parse_array(values))),
    }
}

fn parse_json(json_data: &Value) -> IndexMap<String, CustomValue> {
    match json_data {
        Value::Object(map) => map.iter().fold(IndexMap::new(), |mut acc, (key, val)| {
            let entry = parse_json_value(val);
            acc.insert(key.clone(), entry);
            acc
        }),
        _ => IndexMap::new(),
    }
}

// grab only the unique types from the JSON array
fn parse_array(items: &[Value]) -> CustomValue {
    let mut unique_types = Vec::new(); // not using hashset because we can't hash a CustomValue
    for item in items.iter().map(parse_json_value) {
        if !unique_types.contains(&item) {
            unique_types.push(item);
        }
    }
    if unique_types.len() == 1 {
        unique_types.into_iter().next().unwrap()
    } else {
        CustomValue::UnionTypes(unique_types)
    }
}

fn merge_maps(
    mut map1: IndexMap<String, CustomValue>,
    map2: IndexMap<String, CustomValue>,
) -> IndexMap<String, CustomValue> {
    for (key, value) in map2 {
        if let Some(existing_value) = map1.get_mut(&key) {
            match existing_value {
                CustomValue::UnionTypes(arr) => {
                    if !arr.contains(&value) {
                        arr.push(value);
                    }
                }
                _ => {
                    let arr = vec![existing_value.clone(), value];
                    *existing_value = CustomValue::UnionTypes(arr);
                }
            }
        } else {
            map1.insert(key, value);
        }
    }
    map1
}

fn extract_types_py(value: &CustomValue, field_name: &str) -> Vec<String> {
    match value {
        CustomValue::UnionTypes(_) => {
            todo!()
        }
        CustomValue::JsonArray(_) => {
            todo!()
        }
        CustomValue::JsonObject(_) => {
            // TODO: write the inner object as another data class
            vec![field_name.to_case(Case::Pascal)]
        }
        CustomValue::JsonPrimitive(primitive) => {
            vec![primitive_to_type(primitive, SupportedLanguages::Python).to_string()]
        }
    }
}

fn extract_types_ts(value: &CustomValue, tab_index: usize) -> Vec<String> {
    match value {
        CustomValue::UnionTypes(arr) => {
            let mut types = HashSet::new();
            for item in arr {
                match item {
                    CustomValue::JsonPrimitive(primitive) => {
                        types.insert(
                            primitive_to_type(primitive, SupportedLanguages::Typescript)
                                .to_string(),
                        );
                    }
                    CustomValue::JsonObject(obj) => {
                        types.insert(render_object(
                            obj,
                            false,
                            tab_index + 1,
                            SupportedLanguages::Typescript,
                        ));
                    }
                    CustomValue::JsonArray(arr) => {
                        let extracted_types =
                            extract_types_ts(&CustomValue::JsonArray(arr.clone()), tab_index);
                        let formatted_types = if extracted_types.len() > 1 {
                            format!("({})[]", extracted_types.join(" | "))
                        } else {
                            format!("{}[]", extracted_types.join(""))
                        };
                        types.insert(formatted_types);
                    }
                    _ => {
                        types.insert("any".to_string());
                    }
                }
            }
            types.into_iter().collect()
        }
        CustomValue::JsonArray(arr) => {
            let mut types = HashSet::new();
            for item in arr {
                match item {
                    CustomValue::JsonPrimitive(primitive) => {
                        types.insert(
                            primitive_to_type(primitive, SupportedLanguages::Typescript)
                                .to_string(),
                        );
                    }
                    CustomValue::JsonObject(obj) => {
                        types.insert(render_object(
                            obj,
                            false,
                            tab_index,
                            SupportedLanguages::Typescript,
                        ));
                    }
                    _ => {
                        types.insert("any".to_string());
                    }
                }
            }
            types.into_iter().collect()
        }
        CustomValue::JsonObject(map) => {
            vec![render_object(
                map,
                false,
                tab_index,
                SupportedLanguages::Typescript,
            )]
        }
        CustomValue::JsonPrimitive(primitive) => {
            vec![primitive_to_type(primitive, SupportedLanguages::Typescript).to_string()]
        }
    }
}

fn primitive_to_type(primitive: &JsonPrimitive, language: SupportedLanguages) -> &'static str {
    match language {
        SupportedLanguages::Typescript => match primitive {
            JsonPrimitive::Null => "null",
            JsonPrimitive::String => "string",
            JsonPrimitive::Int | JsonPrimitive::Float => "number",
            JsonPrimitive::Bool => "boolean",
        },
        SupportedLanguages::Python => match primitive {
            JsonPrimitive::Null => "None",
            JsonPrimitive::String => "str",
            JsonPrimitive::Int => "int",
            JsonPrimitive::Float => "float",
            JsonPrimitive::Bool => "bool",
        },
    }
}

fn render_object(
    fields: &IndexMap<String, CustomValue>,
    use_key_for_first_primitive: bool,
    tab_index: usize,
    language: SupportedLanguages,
) -> String {
    let mut content = String::new();
    let mut first_primitive_key_encountered = false;

    // Helper function to generate indentation
    let indent = |level: usize| " ".repeat(level * 4); // 4 spaces per tab level

    let fields_vec: Vec<(&String, Vec<String>)> = fields
        .iter()
        .map(|(field, value)| {
            let types = match language {
                SupportedLanguages::Typescript => extract_types_ts(value, tab_index),
                SupportedLanguages::Python => extract_types_py(value, field),
            };
            let types_str = if types.iter().any(|t| t != "null" && t != "None") {
                types
                    .into_iter()
                    .filter(|t| t != "null" && t != "None")
                    .collect::<Vec<String>>()
            } else {
                types
            };
            (field, types_str)
        })
        .collect();

    for (field, types) in fields_vec {
        let types_str = types.join(" | ");
        let is_optional =
            types.contains(&"null".to_string()) || types.contains(&"None".to_string());

        match language {
            SupportedLanguages::Typescript => {
                let optional_marker = if is_optional { "?" } else { "" };
                if types.len() == 1
                    && (types.contains(&"null".to_string())
                        || types.contains(&"{}".to_string())
                        || types.contains(&"[]".to_string()))
                {
                    content.push_str(&format!(
                        "{}// {}{}: {};\n",
                        indent(tab_index),
                        field,
                        optional_marker,
                        types_str
                    ));
                } else if !first_primitive_key_encountered && use_key_for_first_primitive {
                    content.push_str(&format!(
                        "{}{}{}: Key<{}>;\n",
                        indent(tab_index),
                        field,
                        optional_marker,
                        types_str
                    ));
                    first_primitive_key_encountered = true;
                } else {
                    content.push_str(&format!(
                        "{}{}{}: {};\n",
                        indent(tab_index),
                        field,
                        optional_marker,
                        types_str
                    ));
                }
            }
            SupportedLanguages::Python => {
                let type_str = if is_optional {
                    format!("Optional[{}]", types_str)
                } else {
                    types_str
                };
                content.push_str(&format!("{}{}: {}\n", indent(tab_index), field, type_str));
            }
        }
    }
    match language {
        SupportedLanguages::Typescript if content.trim().is_empty() => "{}".to_string(),
        SupportedLanguages::Typescript => format!("{{\n{}{}}}", content, indent(tab_index - 1)),
        SupportedLanguages::Python => content,
    }
}

fn render_typescript_interface(
    interface_name: &str,
    fields: &IndexMap<String, CustomValue>,
) -> String {
    let mut interface = format!(
        r#"
import {{ Key }} from "@514labs/moose-lib";    
export interface {} "#,
        interface_name
    );
    interface.push_str(&render_object(
        fields,
        true,
        1,
        SupportedLanguages::Typescript,
    ));
    interface
}

fn render_python_dataclass(class_name: &str, fields: &IndexMap<String, CustomValue>) -> String {
    let mut class_def = format!(
        r#"from dataclasses import dataclass
from typing import Optional, Union, Any

@dataclass
class {}:
"#,
        class_name
    );
    class_def.push_str(&render_object(fields, false, 1, SupportedLanguages::Python));
    class_def
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON_CONTENT: &str = r#"
        [
            {
                "id": 1,
                "name": "John Doe",
                "age": 30,
                "is_active": true,
                "scores": [85, 90, 78],
                "address": {
                    "street": "123 Main St",
                    "city": "Anytown"
                }
            },
            {
                "id": 2,
                "name": "Jane Smith",
                "age": 28,
                "is_active": false,
                "scores": [92, 88, 95],
                "address": null
            }
        ]
        "#;

    #[test]
    fn test_json_to_typescript() {
        let result = parse_and_generate(
            "User",
            JSON_CONTENT.to_string(),
            SupportedLanguages::Typescript,
        );

        let expected = r#"
import { Key } from "@514labs/moose-lib";    
export interface User {
    id: Key<number>;
    name: string;
    age: number;
    is_active: boolean;
    scores: number[];
    address: {
        street: string;
        city: string;
    };
}"#;

        print!("{}", result);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_json_to_python() {
        let result =
            parse_and_generate("User", JSON_CONTENT.to_string(), SupportedLanguages::Python);

        let expected = r#"from dataclasses import dataclass
from typing import Optional, Union, Any

@dataclass
class Address:
    street: str
    city: str

@dataclass
class User:
    id: int
    name: str
    age: int
    is_active: bool
    scores: list[int]
    address: Optional[Any]
"#;

        print!("{}", result);
        assert_eq!(result, expected);
    }
}
