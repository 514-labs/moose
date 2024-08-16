use serde::de::{Error, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Serializer as JsonSerializer;
use std::collections::HashMap;
use std::fmt::Formatter;

use crate::framework::core::infrastructure::table::{Column, ColumnType, EnumValue};
use crate::framework::data_model::model::DataModel;

struct State {
    seen: bool,
}

pub struct DataModelVisitor {
    columns: HashMap<String, (Column, State)>,
}

impl DataModelVisitor {
    pub fn new(data_model: &DataModel) -> Self {
        DataModelVisitor {
            columns: data_model
                .columns
                .iter()
                .map(|c| (c.name.clone(), (c.clone(), State { seen: false })))
                .collect(),
        }
    }
}

struct DateString(String);
impl<'de> Deserialize<'de> for DateString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|s| {
            match chrono::DateTime::parse_from_rfc3339(&s) {
                Ok(_) => Ok(DateString(s)),
                Err(_) => Err(D::Error::custom("Invalid date format")),
            }
        })
    }
}
impl Serialize for DateString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        String::serialize(&self.0, serializer)
    }
}

fn get_and_set<
    'de,
    A: MapAccess<'de>,
    S: SerializeMap,
    T: Deserialize<'de> + ?Sized + Serialize,
>(
    map: &mut A,
    map_serializer: &mut S,
    key: String,
    required: bool,
    validation: Option<impl Fn(&T) -> Option<A::Error>>,
) -> Result<(), A::Error> {
    let value = map.next_value::<Option<T>>()?;
    if required && value.is_none() {
        return Err(A::Error::custom(format!(
            "Required value for field {} not found",
            key
        )));
    };

    if let Some(v) = validation {
        if let Some(ref value) = value {
            if let Some(e) = v(value) {
                return Err(e);
            }
        }
    }

    map_serializer
        .serialize_entry(&key, &value)
        .map_err(A::Error::custom)
}

impl<'de> Visitor<'de> for &mut DataModelVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("an object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut vec = Vec::with_capacity(128);
        let mut writer = JsonSerializer::new(&mut vec);
        let mut map_serializer = writer.serialize_map(None).map_err(A::Error::custom)?;

        while let Some(key) = map.next_key::<String>()? {
            if let Some((column, state)) = self.columns.get_mut(&key) {
                state.seen = true;
                match column.data_type {
                    ColumnType::String => {
                        get_and_set::<_, _, String>(
                            &mut map,
                            &mut map_serializer,
                            key,
                            column.required,
                            None::<fn(&_) -> _>,
                        )?;
                    }
                    ColumnType::Boolean => {
                        get_and_set::<_, _, bool>(
                            &mut map,
                            &mut map_serializer,
                            key,
                            column.required,
                            None::<fn(&_) -> _>,
                        )?;
                    }
                    ColumnType::Int => {
                        get_and_set::<_, _, i64>(
                            &mut map,
                            &mut map_serializer,
                            key,
                            column.required,
                            None::<fn(&_) -> _>,
                        )?;
                    }

                    ColumnType::BigInt
                    | ColumnType::Decimal
                    | ColumnType::Json
                    | ColumnType::Bytes => return Err(A::Error::custom("UnsupportedColumnType")),
                    ColumnType::Float => {
                        get_and_set::<_, _, f64>(
                            &mut map,
                            &mut map_serializer,
                            key,
                            column.required,
                            None::<fn(&_) -> _>,
                        )?;
                    }
                    ColumnType::DateTime => {
                        get_and_set::<_, _, DateString>(
                            &mut map,
                            &mut map_serializer,
                            key,
                            column.required,
                            None::<fn(&_) -> _>,
                        )?;
                    }
                    ColumnType::Enum(enum_def) => {
                        get_and_set::<_, _, EnumValue>(
                            &mut map,
                            &mut map_serializer,
                            key,
                            column.required,
                            Some(|v| match v {
                                EnumValue::Int(i) => enum_def.values.iter().any(|v| v.value),
                                EnumValue::String(_) => {}
                            }),
                        )?;
                    }
                    ColumnType::Array(_) => {
                        get_and_set::<_, _, String>(
                            &mut map,
                            &mut map_serializer,
                            key,
                            column.required,
                            None::<fn(&_) -> _>,
                        )?;
                    }
                    ColumnType::Nested(_) => {
                        get_and_set::<_, _, String>(
                            &mut map,
                            &mut map_serializer,
                            key,
                            column.required,
                            None::<fn(&_) -> _>,
                        )?;
                    }
                }
            }
        }
        SerializeMap::end(map_serializer).map_err(A::Error::custom)?;
        let mut missing_fields: Vec<&str> = Vec::new();
        self.columns.values_mut().for_each(|(column, state)| {
            if !state.seen && column.required {
                missing_fields.push(&column.name)
            }
            state.seen = false
        });

        if !missing_fields.is_empty() {
            return Err(A::Error::custom(format!(
                "Missing fields: {}",
                missing_fields.join(", ")
            )));
        }

        Ok(vec)
    }
}
