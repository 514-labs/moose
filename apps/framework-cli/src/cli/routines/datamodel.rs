use log::debug;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::{collections::HashSet, fs};

#[derive(Debug, Clone)]
enum CustomValue {
    TypeArray(Vec<CustomValue>),
    JsonArray(Vec<CustomValue>),
    JsonObject(HashMap<String, CustomValue>),
    JsonPrimitive(Value),
}

// function which reads a json file
pub fn read_json_file(name: String, file_path: String) -> Result<String, std::io::Error> {
    let file = fs::read_to_string(file_path)?;
    println!("done parsing");
    let map = parse_json_file(&file).unwrap();
    let ts_inter: String = render_typescript_interface(&name, &map);
    debug!("{:?}", ts_inter);
    Ok(ts_inter.to_string())
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

fn parse_array(array: &Value) -> Vec<CustomValue> {
    match array {
        Value::Array(items) => items.iter().map(parse_json_value).collect(),
        _ => vec![],
    }
}

fn parse_json_value(json_data: &Value) -> CustomValue {
    match json_data {
        Value::Null => CustomValue::JsonPrimitive(Value::Null),
        Value::String(_) => CustomValue::JsonPrimitive(Value::String("string".to_string())),
        Value::Number(num) => CustomValue::JsonPrimitive(Value::Number(num.clone())),
        Value::Bool(bool) => CustomValue::JsonPrimitive(Value::Bool(bool.clone())),
        Value::Object(_) => CustomValue::JsonObject(parse_json(json_data)),
        Value::Array(_) => CustomValue::JsonArray(parse_array(json_data)),
    }
}

fn parse_json_file(file_content: &str) -> Result<HashMap<String, CustomValue>, serde_json::Error> {
    let json_array: Value = serde_json::from_str(file_content).unwrap();
    let mut schema = HashMap::new();

    let mut i = 0;
    if let Value::Array(array) = json_array {
        for obj in array {
            let parsed_map = parse_json(&obj);
            i += 1;
            if (i % 50 == 0) {
                println!("Processed {} records", i);
            }
            if (i == 3) {
                println!("Processed {} records", i);
                break;
            }
            schema = merge_maps(schema.clone(), parsed_map);
        }
    }

    println!("{:?}", schema);

    Ok(schema)
}

// function which merges two maps, if the key is already present in the first map, it merges the values
// by appending it to an array

fn merge_maps(
    mut map1: HashMap<String, CustomValue>,
    map2: HashMap<String, CustomValue>,
) -> HashMap<String, CustomValue> {
    for (key, value) in map2 {
        if let Some(existing_value) = map1.get_mut(&key) {
            match (existing_value, value) {
                (CustomValue::TypeArray(ref mut existing), CustomValue::TypeArray(mut new)) => {
                    existing.append(&mut new);
                }
                (CustomValue::JsonArray(ref mut existing), CustomValue::JsonArray(mut new)) => {
                    for item in new.drain(..) {
                        if !existing.contains(&item) {
                            existing.push(item);
                        }
                    }
                }
                (CustomValue::JsonObject(ref mut existing), CustomValue::JsonObject(new)) => {
                    let merged_obj = merge_maps(existing.clone(), new);
                    *existing = merged_obj;
                }
                (existing, new) => {
                    let mut arr = vec![existing.clone(), new];
                    *existing = CustomValue::TypeArray(arr);
                }
            }
        } else {
            map1.insert(key, value);
        }
    }
    map1
}

fn extract_types(value: &CustomValue) -> Vec<String> {
    match value {
        CustomValue::TypeArray(arr) => {
            let mut types = HashSet::new();
            for item in arr {
                match item {
                    CustomValue::JsonPrimitive(Value::String(_)) => {
                        types.insert("string".to_string());
                    }
                    CustomValue::JsonPrimitive(Value::Number(_)) => {
                        types.insert("number".to_string());
                    }
                    CustomValue::JsonPrimitive(Value::Null) => {
                        types.insert("null".to_string());
                    }
                    CustomValue::JsonPrimitive(Value::Bool(_)) => {
                        types.insert("boolean".to_string());
                    }
                    CustomValue::JsonPrimitive(Value::Array(_)) => {
                        types.insert("Array".to_string());
                    }
                    CustomValue::JsonPrimitive(Value::Object(_)) => {
                        types.insert("Object".to_string());
                    }
                    CustomValue::JsonObject(obj) => {
                        types.insert(render_typescript_object(obj));
                    }
                    CustomValue::JsonArray(arr) => {
                        types.insert(
                            extract_types(&CustomValue::JsonArray(arr.clone())).join(" | "),
                        );
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
                    CustomValue::JsonPrimitive(Value::String(_)) => {
                        types.insert("string[]".to_string());
                    }
                    CustomValue::JsonPrimitive(Value::Number(_)) => {
                        types.insert("number[]".to_string());
                    }
                    CustomValue::JsonPrimitive(Value::Null) => {
                        types.insert("null".to_string());
                    }
                    CustomValue::JsonPrimitive(Value::Bool(_)) => {
                        types.insert("boolean[]".to_string());
                    }
                    CustomValue::JsonPrimitive(Value::Array(_)) => {
                        types.insert("Array".to_string());
                    }
                    CustomValue::JsonPrimitive(Value::Object(_)) => {
                        types.insert("Object".to_string());
                    }
                    CustomValue::JsonObject(obj) => {
                        types.insert(render_typescript_object(obj));
                    }
                    _ => {
                        types.insert("any".to_string());
                    }
                }
            }
            types.into_iter().collect()
        }
        CustomValue::JsonObject(map) => {
            vec![render_typescript_object(map)]
        }
        CustomValue::JsonPrimitive(Value::Array(_)) => vec!["Array".to_string()],
        CustomValue::JsonPrimitive(Value::Object(_)) => vec!["Object".to_string()],
        CustomValue::JsonPrimitive(Value::String(_)) => vec!["string".to_string()],
        CustomValue::JsonPrimitive(Value::Null) => vec!["null".to_string()],
        CustomValue::JsonPrimitive(Value::Bool(_)) => vec!["boolean".to_string()],
        CustomValue::JsonPrimitive(Value::Number(_)) => vec!["number".to_string()],
    }
}

fn render_typescript_object(fields: &HashMap<String, CustomValue>) -> String {
    let mut object = format!("{{\n");
    for (field, value) in fields {
        let types = extract_types(value);
        let is_optional = types.contains(&"null".to_string());
        let types_str: Vec<String> = types.into_iter().filter(|t| t != "null").collect();
        let types_str = types_str.join(" | ");

        let optional_marker = if is_optional { "?" } else { "" };
        object.push_str(&format!("  {}{}: {};\n", field, optional_marker, types_str));
    }
    object.push_str("}\n");
    object
}

fn render_typescript_interface(
    interface_name: &str,
    fields: &HashMap<String, CustomValue>,
) -> String {
    let mut interface = format!("export interface {} ", interface_name);
    interface.push_str(&render_typescript_object(fields));
    interface
}
