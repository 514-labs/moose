use serde_json::{Deserializer, Value};
use std::collections::HashMap;
use std::io::BufReader;
use std::{collections::HashSet, fs};

use crate::cli::display::{Message, MessageType};
use crate::framework::typescript::generator::InterfaceFieldType;

#[derive(Debug, Clone, PartialEq)] // Add the PartialEq trait
enum CustomValue {
    TypeArray(Vec<CustomValue>),
    JsonArray(Vec<CustomValue>),
    JsonObject(HashMap<String, CustomValue>),
    JsonPrimitive(InterfaceFieldType),
}

// function which reads a json file
pub fn read_json_file(name: String, file_path: String) -> Result<String, std::io::Error> {
    let file = fs::read_to_string(file_path)?;
    let map = parse_json_file(&file).unwrap();
    let ts_inter: String = render_typescript_interface(&name, &map);
    Ok(ts_inter.to_string())
}

fn parse_json_file(file_content: &str) -> Result<HashMap<String, CustomValue>, serde_json::Error> {
    let reader = BufReader::new(file_content.as_bytes());
    let stream = Deserializer::from_reader(reader).into_iter::<Value>();

    let mut schema = HashMap::new();

    let mut i = 0;
    for obj in stream {
        if let Value::Array(array) = obj.unwrap() {
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
                schema = merge_maps(schema.clone(), parsed_map);
            }
        }
    }

    Ok(schema)
}

fn parse_json_value(json_data: &Value) -> CustomValue {
    match json_data {
        Value::Null => CustomValue::JsonPrimitive(InterfaceFieldType::Null),
        Value::String(_) => CustomValue::JsonPrimitive(InterfaceFieldType::String),
        Value::Number(_) => CustomValue::JsonPrimitive(InterfaceFieldType::Number),
        Value::Bool(_) => CustomValue::JsonPrimitive(InterfaceFieldType::Boolean),
        Value::Object(_) => CustomValue::JsonObject(parse_json(json_data)),
        Value::Array(_) => CustomValue::JsonArray(parse_array(json_data)),
    }
}

fn parse_json(json_data: &Value) -> HashMap<String, CustomValue> {
    match json_data {
        Value::Object(map) => map.iter().fold(HashMap::new(), |mut acc, (key, val)| {
            let entry = parse_json_value(val);
            acc.insert(key.clone(), entry);
            acc
        }),
        _ => HashMap::new(),
    }
}

// grab only the unique types from the JSON array
fn parse_array(array: &Value) -> Vec<CustomValue> {
    match array {
        Value::Array(items) => {
            let mut unique_values = Vec::new();
            for item in items.iter().map(parse_json_value) {
                if !unique_values.contains(&item) {
                    unique_values.push(item);
                }
            }
            unique_values
        }
        _ => vec![],
    }
}

fn merge_maps(
    mut map1: HashMap<String, CustomValue>,
    map2: HashMap<String, CustomValue>,
) -> HashMap<String, CustomValue> {
    for (key, value) in map2 {
        if let Some(existing_value) = map1.get_mut(&key) {
            match existing_value {
                CustomValue::TypeArray(arr) => {
                    if !arr.contains(&value) {
                        arr.push(value);
                    }
                }
                _ => {
                    if !matches!(*existing_value, CustomValue::TypeArray(ref _arr)) {
                        let arr = match existing_value {
                            CustomValue::TypeArray(ref mut arr) => {
                                arr.push(value);
                                arr.clone()
                            }
                            _ => vec![existing_value.clone(), value],
                        };
                        *existing_value = CustomValue::TypeArray(arr);
                    }
                }
            }
        } else {
            map1.insert(key, CustomValue::TypeArray(vec![value]));
        }
    }
    map1
}

fn extract_types(value: &CustomValue, tab_index: usize) -> Vec<String> {
    match value {
        CustomValue::TypeArray(arr) => {
            let mut types = HashSet::new();
            for item in arr {
                match item {
                    CustomValue::JsonPrimitive(InterfaceFieldType::String) => {
                        types.insert("string".to_string());
                    }
                    CustomValue::JsonPrimitive(InterfaceFieldType::Number) => {
                        types.insert("number".to_string());
                    }
                    CustomValue::JsonPrimitive(InterfaceFieldType::Null) => {
                        types.insert("null".to_string());
                    }
                    CustomValue::JsonPrimitive(InterfaceFieldType::Boolean) => {
                        types.insert("boolean".to_string());
                    }
                    CustomValue::JsonPrimitive(InterfaceFieldType::Array(_)) => {
                        types.insert("Array".to_string());
                    }
                    CustomValue::JsonPrimitive(InterfaceFieldType::Object(_)) => {
                        types.insert("Object".to_string());
                    }
                    CustomValue::JsonObject(obj) => {
                        types.insert(render_typescript_object(obj, false, tab_index + 1));
                    }
                    CustomValue::JsonArray(arr) => {
                        let extracted_types =
                            extract_types(&CustomValue::JsonArray(arr.clone()), tab_index);
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
                    CustomValue::JsonPrimitive(InterfaceFieldType::String) => {
                        types.insert("string".to_string());
                    }
                    CustomValue::JsonPrimitive(InterfaceFieldType::Number) => {
                        types.insert("number".to_string());
                    }
                    CustomValue::JsonPrimitive(InterfaceFieldType::Null) => {
                        types.insert("null".to_string());
                    }
                    CustomValue::JsonPrimitive(InterfaceFieldType::Boolean) => {
                        types.insert("boolean".to_string());
                    }
                    CustomValue::JsonObject(obj) => {
                        types.insert(render_typescript_object(obj, false, tab_index));
                    }
                    _ => {
                        types.insert("any".to_string());
                    }
                }
            }
            types.into_iter().collect()
        }
        CustomValue::JsonObject(map) => {
            vec![render_typescript_object(map, false, tab_index)]
        }
        CustomValue::JsonPrimitive(InterfaceFieldType::Array(_)) => vec!["Array".to_string()],
        CustomValue::JsonPrimitive(InterfaceFieldType::Object(_)) => vec!["Object".to_string()],
        CustomValue::JsonPrimitive(InterfaceFieldType::String) => vec!["string".to_string()],
        CustomValue::JsonPrimitive(InterfaceFieldType::Null) => vec!["null".to_string()],
        CustomValue::JsonPrimitive(InterfaceFieldType::Boolean) => vec!["boolean".to_string()],
        CustomValue::JsonPrimitive(InterfaceFieldType::Number) => vec!["number".to_string()],
        _ => vec!["unsupported".to_string()],
    }
}

fn render_typescript_object(
    fields: &HashMap<String, CustomValue>,
    use_key_for_first_primitive: bool,
    tab_index: usize,
) -> String {
    let mut content = String::new();
    let mut first_primitive_key_encountered = false;

    // Helper function to generate indentation
    let indent = |level: usize| " ".repeat(level * 4); // 4 spaces per tab level

    let mut fields_vec: Vec<(&String, Vec<String>)> = fields
        .iter()
        .map(|(field, value)| {
            let types = extract_types(value, tab_index);
            let types_str = if types.iter().any(|t| t != "null") {
                types
                    .into_iter()
                    .filter(|t| t != "null")
                    .collect::<Vec<String>>()
            } else {
                types
            };
            (field, types_str)
        })
        .collect();
    fields_vec.sort_by(|a, b| a.0.cmp(b.0));
    for (field, types) in fields_vec {
        let types_str = types.join(" | ");
        let is_optional = types.contains(&"null".to_string());
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
    // Check if the content is empty
    if content.trim().is_empty() {
        "{}".to_string()
    } else {
        format!(
            "{{{}\n{}{}}}",
            indent(tab_index),
            content,
            indent(tab_index - 1)
        )
    }
}

fn render_typescript_interface(
    interface_name: &str,
    fields: &HashMap<String, CustomValue>,
) -> String {
    let mut interface = format!(
        r#"
import {{ Key }} from "@514labs/moose-lib";    
export interface {}"#,
        interface_name
    );
    interface.push_str(&render_typescript_object(fields, true, 1));
    interface
}
