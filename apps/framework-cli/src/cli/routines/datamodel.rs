use log::debug;
use serde_json::Value;
use std::collections::HashMap;
use std::{collections::HashSet, fs, io::Write};

use crate::framework::typescript::generator::{
    InterfaceField, InterfaceFieldType, TypescriptInterface,
};
use crate::framework::typescript::templates::{render_interface, TypescriptRenderingError};

use super::{RoutineFailure, RoutineSuccess};

// function which reads a json file
pub fn read_json_file(name: String, file_path: String) -> Result<String, std::io::Error> {
    let file = fs::read_to_string(file_path)?;
    let map = parse_json_file(name, &file).unwrap();
    debug!("{:?}", map);
    Ok(map)
}

fn parse_json(json_data: &Value) -> HashMap<String, HashSet<InterfaceFieldType>> {
    match json_data {
        Value::Object(map) => map.iter().fold(HashMap::new(), |mut acc, (key, val)| {
            let entry = match val {
                Value::Null => HashSet::from([InterfaceFieldType::Null]),
                Value::String(_) => HashSet::from([InterfaceFieldType::String]),
                Value::Number(_) => HashSet::from([InterfaceFieldType::Number]),
                Value::Bool(_) => HashSet::from([InterfaceFieldType::Boolean]),
                Value::Object(_) => {
                    let nested_map = parse_json(val);
                    HashSet::from([InterfaceFieldType::Object(Box::new(
                        hashmap_to_typescript_interface(key.to_string(), nested_map),
                    ))])
                }
                Value::Array(arr) => {
                    let mut field_types = HashSet::new();
                    for item in arr {
                        let nested_map = parse_json(item);
                        field_types.insert(InterfaceFieldType::Array(Box::new(
                            InterfaceFieldType::Object(Box::new(hashmap_to_typescript_interface(
                                key.to_string(),
                                nested_map,
                            ))),
                        )));
                    }
                    field_types
                }
            };
            acc.entry(key.clone())
                .or_insert_with(HashSet::new)
                .extend(entry);
            acc
        }),
        Value::Array(array) => array.iter().fold(HashMap::new(), |mut acc, item| {
            let item_map = parse_json(item);
            for (key, val) in item_map {
                acc.entry(key).or_insert_with(HashSet::new).extend(val);
            }
            acc
        }),
        _ => HashMap::new(),
    }
}

fn merge_maps(
    map1: &mut HashMap<String, HashSet<InterfaceFieldType>>,
    map2: HashMap<String, HashSet<InterfaceFieldType>>,
) {
    for (key, val) in map2 {
        map1.entry(key).or_insert_with(HashSet::new).extend(val);
    }
}

fn hashmap_to_typescript_interface(
    name: String,
    map: HashMap<String, HashSet<InterfaceFieldType>>,
) -> TypescriptInterface {
    let fields = map
        .into_iter()
        .map(|(key, types)| InterfaceField {
            name: key,
            comment: None,
            is_optional: false,
            field_type: InterfaceFieldType::from_types(types),
        })
        .collect();

    TypescriptInterface { name, fields }
}

impl InterfaceFieldType {
    fn from_types(types: HashSet<InterfaceFieldType>) -> Self {
        if types.len() == 1 {
            types.into_iter().next().unwrap()
        } else {
            InterfaceFieldType::String
        }
    }
}

fn parse_json_file(name: String, file_content: &str) -> Result<String, TypescriptRenderingError> {
    let json_array: Value = serde_json::from_str(file_content).unwrap();
    let mut schema = HashMap::new();

    let mut i = 0;
    if let Value::Array(array) = json_array {
        for obj in array {
            let parsed_map = parse_json(&obj);
            i += 1;
            if (i % 1000 == 0) {
                println!("Processed {} records", i);
            }
            merge_maps(&mut schema, parsed_map);
        }
    }

    let cloned_schema = schema.clone();
    println!("Schema: {:?}", cloned_schema);
    let interface = hashmap_to_typescript_interface(name, cloned_schema);

    let interface_res = render_interface(&interface);

    return interface_res;
}
