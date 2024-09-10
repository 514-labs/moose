use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::marker::PhantomData;

use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserializer, Serialize, Serializer};
use serde_json::Serializer as JsonSerializer;

use crate::framework::core::infrastructure::table::{Column, ColumnType};
use crate::framework::data_model::model::DataModel;

struct State {
    seen: bool,
}

trait SerializeValue {
    type Error: serde::ser::Error;

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize;
}
impl<S> SerializeValue for S
where
    S: SerializeMap,
{
    type Error = S::Error;

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // must have called serialize_key first
        S::serialize_value(self, value)
    }
}

struct DummyWrapper<'a, T>(&'a mut T); // workaround so that implementations don't clash
impl<'a, S> SerializeValue for DummyWrapper<'a, S>
where
    S: SerializeSeq,
{
    type Error = S::Error;

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.0.serialize_element(value)
    }
}

struct ValueVisitor<'a, S: SerializeValue> {
    t: &'a ColumnType,
    required: bool,
    write_to: &'a mut S,
}
impl<'de, 'a, S: SerializeValue> DeserializeSeed<'de> for &mut ValueVisitor<'a, S> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}
impl<'de, 'a, S: SerializeValue> Visitor<'de> for &mut ValueVisitor<'a, S> {
    type Value = ();

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        match self.t {
            ColumnType::Boolean => formatter.write_str("a boolean value"),
            ColumnType::Int => formatter.write_str("an integer value"),
            ColumnType::Float => formatter.write_str("a floating-point value"),
            ColumnType::String => formatter.write_str("a string value"),
            ColumnType::DateTime => formatter.write_str("a datetime value"),
            ColumnType::Enum(_) => formatter.write_str("an enum value"),
            ColumnType::Array(_) => formatter.write_str("an array value"),
            ColumnType::Nested(_) => formatter.write_str("a nested object"),

            ColumnType::BigInt | ColumnType::Decimal | ColumnType::Json | ColumnType::Bytes => {
                formatter.write_str("a value matching the column type")
            }
        }
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.t {
            ColumnType::Boolean => self.write_to.serialize_value(&v).map_err(Error::custom),
            _ => Err(Error::invalid_type(serde::de::Unexpected::Bool(v), &self)),
        }
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.t {
            ColumnType::Int => self.write_to.serialize_value(&v).map_err(Error::custom),
            _ => Err(Error::invalid_type(serde::de::Unexpected::Signed(v), &self)),
        }
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.t {
            ColumnType::Int => self.write_to.serialize_value(&v).map_err(Error::custom),
            _ => Err(Error::invalid_type(
                serde::de::Unexpected::Unsigned(v),
                &self,
            )),
        }
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.t {
            ColumnType::Float => self.write_to.serialize_value(&v).map_err(Error::custom),
            _ => Err(Error::invalid_type(serde::de::Unexpected::Float(v), &self)),
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.t {
            ColumnType::String => self.write_to.serialize_value(v).map_err(Error::custom),
            ColumnType::DateTime => {
                chrono::DateTime::parse_from_rfc3339(v)
                    .map_err(|_| E::custom("Invalid date format"))?;

                self.write_to.serialize_value(v).map_err(Error::custom)
            }
            ColumnType::Enum(ref _enum_def) => {
                // TODO: Implement enum validation
                self.write_to.serialize_value(v).map_err(Error::custom)
            }
            _ => Err(Error::invalid_type(serde::de::Unexpected::Str(v), &self)),
        }
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if self.required {
            return Err(E::custom("Required value, but is none".to_string()));
        }
        self.write_to
            .serialize_value(&None::<bool>)
            .map_err(Error::custom)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        match self.t {
            ColumnType::Array(ref inner_type) => {
                self.write_to
                    .serialize_value(&SeqAccessSerializer {
                        inner_type,
                        seq: RefCell::new(&mut seq),
                        _phantom_data: &PHANTOM_DATA,
                    })
                    .map_err(A::Error::custom)?;
                Ok(())
            }
            _ => Err(Error::invalid_type(serde::de::Unexpected::Seq, &self)),
        }
    }
}

impl<'a, 'de, A: SeqAccess<'de>> Serialize for SeqAccessSerializer<'a, 'de, A> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut write_to = serializer.serialize_seq(None)?;
        let mut visitor = ValueVisitor {
            t: self.inner_type,
            required: true,
            write_to: &mut DummyWrapper(&mut write_to),
        };
        let mut seq = self.seq.borrow_mut();
        while let Some(()) = (*seq)
            .next_element_seed(&mut visitor)
            .map_err(serde::ser::Error::custom)?
        {}
        write_to.end()
    }
}

static PHANTOM_DATA: PhantomData<()> = PhantomData {};
struct SeqAccessSerializer<'a, 'de, A: SeqAccess<'de>> {
    inner_type: &'a ColumnType,
    seq: RefCell<&'a mut A>,
    _phantom_data: &'de PhantomData<()>,
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

// struct UnwrappedEnumValue(EnumValue);
// impl Serialize for UnwrappedEnumValue {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         match self.0 {
//             EnumValue::Int(i) => serializer.serialize_u8(i),
//             EnumValue::String(ref s) => serializer.serialize_str(s),
//         }
//     }
// }
// impl<'de> Deserialize<'de> for UnwrappedEnumValue {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         deserializer.deserialize_any(UnwrappedEnumValueVisitor)
//     }
// }
// struct UnwrappedEnumValueVisitor;
// impl<'de> Visitor<'de> for UnwrappedEnumValueVisitor {
//     type Value = UnwrappedEnumValue;

//     fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
//         todo!()
//     }

//     fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         Ok(UnwrappedEnumValue(EnumValue::Int(v)))
//     }

//     fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
//     where
//         E: Error,
//     {
//         Ok(UnwrappedEnumValue(EnumValue::String(v.to_string())))
//     }
// }

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

                map_serializer
                    .serialize_key(&key)
                    .map_err(A::Error::custom)?;

                let mut visitor = ValueVisitor {
                    t: &column.data_type,
                    write_to: &mut map_serializer,
                    required: column.required,
                };
                map.next_value_seed(&mut visitor)?;
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
//
// // TODO: cache this closure
// fn to_validation<'de, A: MapAccess<'de>>(
//     data_enum: &DataEnum,
// ) -> impl Fn(&EnumValue) -> Option<A::Error> {
//     // match &data_enum.values[0].value {
//     //     EnumValue::Int(_) => {
//     //         let values = data_enum.values.iter().map(|v| match v.value {
//     //             EnumValue::Int(i) => i,
//     //             EnumValue::String(_) => {
//     //                 panic!("ahhhh")
//     //             }
//     //         })
//     //     },
//     //     EnumValue::String(_) => {}
//     // };
//     |enum_value: &EnumValue| None
// }

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::framework::data_model::config::DataModelConfig;

    use super::*;

    #[test]
    fn test_happy_path_all_types() {
        let data_model = DataModel {
            columns: vec![
                Column {
                    name: "string_col".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
                Column {
                    name: "int_col".to_string(),
                    data_type: ColumnType::Int,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
                Column {
                    name: "float_col".to_string(),
                    data_type: ColumnType::Float,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
                Column {
                    name: "bool_col".to_string(),
                    data_type: ColumnType::Boolean,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
                Column {
                    name: "date_col".to_string(),
                    data_type: ColumnType::DateTime,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                },
            ],
            name: "TestModel".to_string(),
            config: DataModelConfig::default(),
            abs_file_path: PathBuf::from("/path/to/test_model.rs"),
            version: "1.0".to_string(),
        };

        let json = r#"
        {
            "string_col": "test",
            "int_col": 42,
            "float_col": 3.14,
            "bool_col": true,
            "date_col": "2024-09-10T17:34:51+00:00"
        }
        "#;

        let result = serde_json::Deserializer::from_str(json)
            .deserialize_any(&mut DataModelVisitor::new(&data_model))
            .unwrap();

        let expected = r#"{"string_col":"test","int_col":42,"float_col":3.14,"bool_col":true,"date_col":"2024-09-10T17:34:51+00:00"}"#;

        assert_eq!(String::from_utf8(result), Ok(expected.to_string()));
    }

    #[test]
    fn test_bad_date_format() {
        let data_model = DataModel {
            columns: vec![Column {
                name: "date_col".to_string(),
                data_type: ColumnType::DateTime,
                required: true,
                unique: false,
                primary_key: false,
                default: None,
            }],
            name: "TestModel".to_string(),
            config: DataModelConfig::default(),
            abs_file_path: PathBuf::from("/path/to/test_model.rs"),
            version: "1.0".to_string(),
        };

        let json = r#"
        {
            "date_col": "2024-09-10"
        }
        "#;

        let result = serde_json::Deserializer::from_str(json)
            .deserialize_any(&mut DataModelVisitor::new(&data_model));

        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn test_array() {
        let data_model = DataModel {
            columns: vec![Column {
                name: "array_col".to_string(),
                data_type: ColumnType::Array(Box::new(ColumnType::Int)),
                required: true,
                unique: false,
                primary_key: false,
                default: None,
            }],
            name: "TestModel".to_string(),
            config: DataModelConfig::default(),
            abs_file_path: PathBuf::from("/path/to/test_model.rs"),
            version: "1.0".to_string(),
        };

        let json = r#"
        {
            "array_col": [1, 2, 3, 4, 5]
        }
        "#;

        let result = serde_json::Deserializer::from_str(json)
            .deserialize_any(&mut DataModelVisitor::new(&data_model))
            .unwrap();

        let expected = r#"{"array_col":[1,2,3,4,5]}"#;

        assert_eq!(String::from_utf8(result), Ok(expected.to_string()));
    }
}
