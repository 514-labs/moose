use crate::cli::display::{Message, MessageType};
use crate::framework::languages::SupportedLanguages;
use convert_case::{Case, Casing};
use indexmap::IndexMap;
use itertools::Itertools;
use serde_json::{Deserializer, Value};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::io::BufReader;

#[derive(Debug, Clone, PartialEq, Eq)]
enum CustomValue {
    UnionTypes(Vec<CustomValue>),
    // if the array is (which we don't really support),
    // then the inner CustomValue is UnionTypes
    JsonArray(Box<CustomValue>),
    JsonObject(IndexMap<String, CustomValue>),
    JsonPrimitive(JsonPrimitive),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum JsonPrimitive {
    Null,
    String,
    Int,
    Float,
    Bool,
}
pub fn parse_and_generate(name: &str, file: String, language: SupportedLanguages) -> String {
    let map = parse_json_file(&file).unwrap();
    // TODO: better map merging
    println!("{:?}", map);

    match language {
        SupportedLanguages::Typescript => render_typescript_file(name, &map),
        SupportedLanguages::Python => render_python_file(name, &map),
    }
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
    if unique_types.is_empty() {
        CustomValue::JsonPrimitive(JsonPrimitive::Null)
    } else if unique_types.len() == 1 {
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
                    if *existing_value != value {
                        let arr = vec![existing_value.clone(), value];
                        *existing_value = CustomValue::UnionTypes(arr);
                    }
                }
            }
        } else {
            map1.insert(key, value);
        }
    }
    map1
}

fn extract_types_py(
    value: &CustomValue,
    field_name: &str,
    extra_data_classes: &mut IndexMap<String, String>,
) -> String {
    fn is_simple_optional(types: &[CustomValue]) -> bool {
        types.len() == 2 && types.contains(&CustomValue::JsonPrimitive(JsonPrimitive::Null))
    }

    match value {
        CustomValue::UnionTypes(types) if is_simple_optional(types) => {
            let t = types
                .iter()
                .find(|t| **t != CustomValue::JsonPrimitive(JsonPrimitive::Null))
                .unwrap();
            format!(
                "Optional[{}]",
                extract_types_py(t, field_name, extra_data_classes)
            )
        }
        CustomValue::UnionTypes(types) => {
            let mut type_strings = HashSet::new();
            for item in types {
                type_strings.insert(extract_types_py(item, field_name, extra_data_classes));
            }
            format!("Union[{}]", type_strings.into_iter().join(", "))
        }
        CustomValue::JsonArray(inner) => {
            format!(
                "list[{}]",
                extract_types_py(inner, field_name, extra_data_classes)
            )
        }
        CustomValue::JsonObject(fields) => {
            let inner_type_name = field_name.to_case(Case::Pascal);
            if !extra_data_classes.contains_key(&inner_type_name) {
                let nested =
                    render_python_dataclass(&inner_type_name, fields, false, extra_data_classes);
                extra_data_classes.insert(inner_type_name.clone(), nested);
            }
            inner_type_name
        }
        CustomValue::JsonPrimitive(primitive) => {
            primitive_to_type(*primitive, SupportedLanguages::Python).to_string()
        }
    }
}

// returning a Vec because we want to remove parenthesis for JsonArray
fn type_to_string_ts(value: &CustomValue, tab_index: usize) -> Vec<String> {
    match value {
        CustomValue::UnionTypes(types) => {
            let mut type_strings = HashSet::new();
            for item in types {
                for extracted_type in type_to_string_ts(item, tab_index + 1) {
                    type_strings.insert(extracted_type);
                }
            }
            type_strings.into_iter().collect()
        }
        CustomValue::JsonArray(inner) => {
            let extracted_types = type_to_string_ts(inner, tab_index);
            let res = match extracted_types.len().cmp(&1) {
                Ordering::Greater => format!("({})[]", extracted_types.join(" | ")),
                Ordering::Equal => format!("{}[]", extracted_types.first().unwrap()),
                Ordering::Less => unreachable!(),
            };
            vec![res]
        }
        CustomValue::JsonObject(map) => {
            vec![render_object_ts(map, false, tab_index)]
        }
        CustomValue::JsonPrimitive(primitive) => {
            vec![primitive_to_type(*primitive, SupportedLanguages::Typescript).to_string()]
        }
    }
}

fn primitive_to_type(primitive: JsonPrimitive, language: SupportedLanguages) -> &'static str {
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

fn render_object_ts(
    fields: &IndexMap<String, CustomValue>,
    use_key_for_first_primitive: bool,
    tab_index: usize,
) -> String {
    let mut content = String::new();
    let mut first_primitive_key_encountered = false;

    // Helper function to generate indentation
    let indent = |level: usize| " ".repeat(level * 4); // 4 spaces per tab level

    let fields_vec = fields.iter().map(|(field, value)| {
        let types = type_to_string_ts(value, tab_index);
        let types_optional = if types.iter().any(|t| t == "null") {
            (
                types
                    .into_iter()
                    .filter(|t| t != "null")
                    .collect::<Vec<String>>(),
                true,
            )
        } else {
            (types, false)
        };
        (field, types_optional)
    });

    for (field, (types, is_optional)) in fields_vec {
        let types_str = types.join(" | ");

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

    if content.trim().is_empty() {
        "{}".to_string()
    } else {
        format!("{{\n{}{}}}", content, indent(tab_index - 1))
    }
}

fn render_typescript_file(interface_name: &str, fields: &IndexMap<String, CustomValue>) -> String {
    let mut interface = format!(
        r#"
import {{ Key }} from "@514labs/moose-lib";
export interface {} "#,
        interface_name
    );
    interface.push_str(&render_object_ts(fields, true, 1));
    interface
}

fn render_python_dataclass(
    class_name: &str,
    fields: &IndexMap<String, CustomValue>,
    use_key_for_first_primitive: bool,
    extra_data_classes: &mut IndexMap<String, String>,
) -> String {
    let mut class_def = format!(
        r#"
@dataclass
class {}:
"#,
        class_name
    );

    let mut first_primitive_key_encountered = false;
    for (field, t) in fields {
        let mut type_str = extract_types_py(t, field, extra_data_classes);

        if use_key_for_first_primitive
            && (type_str == "str" || type_str == "int")
            && !first_primitive_key_encountered
        {
            type_str = format!("Key[{}]", type_str);
            first_primitive_key_encountered = true;
        }
        class_def.push_str(&format!("    {}: {}\n", field, type_str));
    }
    class_def
}

fn render_python_file(class_name: &str, fields: &IndexMap<String, CustomValue>) -> String {
    let mut class_def = r#"from moose_lib import Key
from dataclasses import dataclass
from typing import Optional, Union, Any
"#
    .to_string();
    let mut extra_data_classes = IndexMap::new();
    let data_model = render_python_dataclass(class_name, fields, true, &mut extra_data_classes);

    for data_class in extra_data_classes.values() {
        class_def.push_str(data_class);
    }
    class_def.push_str(&data_model);

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
                "age": null,
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
    age?: number;
    is_active: boolean;
    scores: number[];
    address?: {
        street: string;
        city: string;
    };
}"#;

        println!("{}", result);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_json_to_python() {
        let result =
            parse_and_generate("User", JSON_CONTENT.to_string(), SupportedLanguages::Python);

        let expected = r#"from moose_lib import Key
from dataclasses import dataclass
from typing import Optional, Union, Any

@dataclass
class Address:
    street: str
    city: str

@dataclass
class User:
    id: Key[int]
    name: str
    age: Optional[int]
    is_active: bool
    scores: list[int]
    address: Optional[Address]
"#;

        println!("{}", result);
        assert_eq!(result, expected);
    }
}
