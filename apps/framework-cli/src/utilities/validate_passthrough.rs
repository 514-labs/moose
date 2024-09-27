use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserializer, Serialize, Serializer};
use serde_json::Serializer as JsonSerializer;

use crate::framework::core::infrastructure::table::{Column, ColumnType, DataEnum, EnumValue};

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

trait EnumInt {
    fn from_u8(u8: u8) -> Self;
    fn from_usize(usize: usize) -> Self;
}
impl EnumInt for u64 {
    fn from_u8(u8: u8) -> u64 {
        u8 as u64
    }
    fn from_usize(usize: usize) -> Self {
        usize as u64
    }
}
impl EnumInt for i64 {
    fn from_u8(u8: u8) -> i64 {
        u8 as i64
    }
    fn from_usize(usize: usize) -> i64 {
        usize as i64
    }
}

fn handle_enum_value<S: SerializeValue, E, T>(
    write_to: &mut S,
    enum_def: &DataEnum,
    v: T,
) -> Result<(), E>
where
    E: Error,
    T: Copy + PartialEq + EnumInt + Serialize + Display,
{
    if enum_def
        .values
        .iter()
        .enumerate()
        .any(|(i, ev)| match &ev.value {
            EnumValue::Int(value) => (T::from_u8(*value)) == v,
            // TODO: string enums have range 1..=length
            // we can skip the iteration
            EnumValue::String(_) => (T::from_usize(i)) == v,
        })
    {
        write_to.serialize_value(&v).map_err(E::custom)
    } else {
        Err(E::custom(format!("Invalid enum value: {}", v)))
    }
}

#[derive(Clone, Copy)] // just two pointers, copying is fine
struct ParentContext<'a> {
    parent: Option<&'a ParentContext<'a>>,
    field_name: &'a str,
}

struct ValueVisitor<'a, S: SerializeValue> {
    t: &'a ColumnType,
    required: bool,
    write_to: &'a mut S,
    context: ParentContext<'a>,
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
        write!(formatter, "at {}, ", self.get_path())?;
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
            ColumnType::Float => self.write_to.serialize_value(&v).map_err(Error::custom),
            ColumnType::Enum(enum_def) => handle_enum_value(self.write_to, enum_def, v),
            _ => Err(Error::invalid_type(serde::de::Unexpected::Signed(v), &self)),
        }
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.t {
            ColumnType::Int => self.write_to.serialize_value(&v).map_err(Error::custom),
            ColumnType::Float => self.write_to.serialize_value(&v).map_err(Error::custom),
            ColumnType::Enum(enum_def) => handle_enum_value(self.write_to, enum_def, v),
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
                chrono::DateTime::parse_from_rfc3339(v).map_err(|_| {
                    E::custom(format!("Invalid date format at {}", self.get_path()))
                })?;

                self.write_to.serialize_value(v).map_err(Error::custom)
            }
            ColumnType::Enum(ref enum_def) => {
                if enum_def.values.iter().any(|ev| match &ev.value {
                    EnumValue::Int(_) => ev.name == v,
                    EnumValue::String(enum_value) => enum_value == v,
                }) {
                    self.write_to.serialize_value(v).map_err(Error::custom)
                } else {
                    Err(E::custom(format!(
                        "Invalid enum value at {}: {}",
                        self.get_path(),
                        v
                    )))
                }
            }
            _ => Err(Error::invalid_type(serde::de::Unexpected::Str(v), &self)),
        }
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if self.required {
            return Err(E::custom(format!(
                "Required value at {}, but is none",
                self.get_path()
            )));
        }
        self.write_to
            // type param of the None does not matter
            // we're writing null anyway
            .serialize_value(&None::<bool>)
            .map_err(Error::custom)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_none()
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        match self.t {
            ColumnType::Array(ref inner_type) => {
                self.write_to
                    .serialize_value(&SeqAccessSerializer {
                        inner_type,
                        seq: RefCell::new(seq),
                        _phantom_data: &PHANTOM_DATA,
                        context: &self.context,
                    })
                    .map_err(A::Error::custom)?;
                Ok(())
            }
            _ => Err(Error::invalid_type(serde::de::Unexpected::Seq, &self)),
        }
    }
    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        match self.t {
            ColumnType::Nested(ref fields) => {
                let inner = DataModelVisitor::with_context(&fields.columns, Some(&self.context));
                let serializer = MapAccessSerializer {
                    inner: RefCell::new(inner),
                    map: RefCell::new(map),
                    _phantom_data: &PHANTOM_DATA,
                };
                self.write_to
                    .serialize_value(&serializer)
                    .map_err(A::Error::custom)
            }
            _ => Err(A::Error::invalid_type(serde::de::Unexpected::Map, &self)),
        }
    }
}

impl<'a, S: SerializeValue> ValueVisitor<'a, S> {
    fn get_path(&self) -> String {
        add_path_component(
            parent_context_to_string(self.context.parent),
            self.context.field_name,
        )
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
            context: *self.context,
        };
        let mut seq = self.seq.borrow_mut();
        while let Some(()) = seq
            .next_element_seed(&mut visitor)
            .map_err(serde::ser::Error::custom)?
        {}
        write_to.end()
    }
}

static PHANTOM_DATA: PhantomData<()> = PhantomData {};
// RefCell for interior mutability
// generally serialization for T should not change the T
// but here we read elements/entries off from the SeqAccess/MapAccess
// as we put it into the output JSON
struct SeqAccessSerializer<'a, 'de, A: SeqAccess<'de>> {
    inner_type: &'a ColumnType,
    seq: RefCell<A>,
    _phantom_data: &'de PhantomData<()>,
    context: &'a ParentContext<'a>,
}
struct MapAccessSerializer<'de, 'a, A: MapAccess<'de>> {
    inner: RefCell<DataModelVisitor<'a>>,
    map: RefCell<A>,
    _phantom_data: &'de PhantomData<()>,
}

impl<'de, 'a, A: MapAccess<'de>> Serialize for MapAccessSerializer<'de, 'a, A> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut write_to = serializer.serialize_map(None)?;
        let map: &mut A = &mut self.map.borrow_mut();
        self.inner
            .borrow_mut()
            .transfer_map_access_to_serialize_map(map, &mut write_to)
            .map_err(serde::ser::Error::custom)?;
        write_to.end()
    }
}

pub struct DataModelVisitor<'a> {
    columns: HashMap<String, (Column, State)>,
    parent_context: Option<&'a ParentContext<'a>>,
}
impl<'a> DataModelVisitor<'a> {
    pub fn new(columns: &[Column]) -> Self {
        Self::with_context(columns, None)
    }
    fn with_context(columns: &[Column], parent_context: Option<&'a ParentContext<'a>>) -> Self {
        DataModelVisitor {
            columns: columns
                .iter()
                .map(|c| (c.name.clone(), (c.clone(), State { seen: false })))
                .collect(),
            parent_context,
        }
    }

    fn transfer_map_access_to_serialize_map<'de, A: MapAccess<'de>, S: SerializeMap>(
        &mut self,
        map: &mut A,
        map_serializer: &mut S,
    ) -> Result<(), A::Error> {
        while let Some(key) = map.next_key::<String>()? {
            if let Some((column, state)) = self.columns.get_mut(&key) {
                state.seen = true;

                map_serializer
                    .serialize_key(&key)
                    .map_err(A::Error::custom)?;

                let mut visitor = ValueVisitor {
                    t: &column.data_type,
                    write_to: map_serializer,
                    required: column.required,
                    context: ParentContext {
                        parent: self.parent_context,
                        field_name: &key,
                    },
                };
                map.next_value_seed(&mut visitor)?;
            } else {
                map.next_value::<serde::de::IgnoredAny>()?;
            }
        }
        let mut missing_fields: Vec<String> = Vec::new();
        self.columns.values_mut().for_each(|(column, state)| {
            if !state.seen && column.required {
                let parent_path = parent_context_to_string(self.parent_context);
                let path = add_path_component(parent_path, &column.name);
                missing_fields.push(path);
            }
            state.seen = false
        });

        if !missing_fields.is_empty() {
            return Err(A::Error::custom(format!(
                "Missing fields: {}",
                missing_fields.join(", ")
            )));
        }

        Ok(())
    }
}

fn parent_context_to_string(parent_context: Option<&ParentContext>) -> String {
    match parent_context {
        Some(ParentContext { parent, field_name }) => {
            add_path_component(parent_context_to_string(*parent), field_name)
        }
        None => String::new(),
    }
}

impl<'de, 'a> Visitor<'de> for &mut DataModelVisitor<'a> {
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

        self.transfer_map_access_to_serialize_map(&mut map, &mut map_serializer)?;
        SerializeMap::end(map_serializer).map_err(A::Error::custom)?;

        Ok(vec)
    }
}
impl<'de, 'a> DeserializeSeed<'de> for &mut DataModelVisitor<'a> {
    type Value = Vec<u8>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}
pub struct DataModelArrayVisitor<'a> {
    pub inner: DataModelVisitor<'a>,
}
impl<'de, 'a> Visitor<'de> for &mut DataModelArrayVisitor<'a> {
    type Value = Vec<Vec<u8>>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("an array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut result = vec![];
        while let Some(element) = seq.next_element_seed(&mut self.inner)? {
            result.push(element)
        }
        Ok(result)
    }
}

fn add_path_component(mut path: String, field_name: &str) -> String {
    if !path.is_empty() {
        path.push('.');
    }
    path.push_str(field_name);
    path
}

#[cfg(test)]
mod tests {
    use crate::framework::core::infrastructure::table::{DataEnum, EnumMember, Nested};

    use super::*;

    #[test]
    fn test_happy_path_all_types() {
        let columns = vec![
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
        ];

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
            .deserialize_any(&mut DataModelVisitor::new(&columns))
            .unwrap();

        let expected = r#"{"string_col":"test","int_col":42,"float_col":3.14,"bool_col":true,"date_col":"2024-09-10T17:34:51+00:00"}"#;

        assert_eq!(String::from_utf8(result), Ok(expected.to_string()));
    }

    #[test]
    fn test_bad_date_format() {
        let columns = vec![Column {
            name: "date_col".to_string(),
            data_type: ColumnType::DateTime,
            required: true,
            unique: false,
            primary_key: false,
            default: None,
        }];

        let json = r#"
        {
            "date_col": "2024-09-10"
        }
        "#;

        let result = serde_json::Deserializer::from_str(json)
            .deserialize_any(&mut DataModelVisitor::new(&columns));

        println!("{:?}", result);
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .contains("Invalid date format at date_col"));
    }

    #[test]
    fn test_array() {
        let columns = vec![Column {
            name: "array_col".to_string(),
            data_type: ColumnType::Array(Box::new(ColumnType::Int)),
            required: true,
            unique: false,
            primary_key: false,
            default: None,
        }];

        let json = r#"
        {
            "array_col": [1, 2, 3, 4, 5]
        }
        "#;

        let result = serde_json::Deserializer::from_str(json)
            .deserialize_any(&mut DataModelVisitor::new(&columns))
            .unwrap();

        let expected = r#"{"array_col":[1,2,3,4,5]}"#;

        assert_eq!(String::from_utf8(result), Ok(expected.to_string()));
    }

    #[test]
    fn test_enum_valid_and_invalid() {
        let columns = vec![Column {
            name: "enum_col".to_string(),
            data_type: ColumnType::Enum(DataEnum {
                name: "TestEnum".to_string(),
                values: vec![
                    EnumMember {
                        name: "Option1".to_string(),
                        value: EnumValue::String("option1".to_string()),
                    },
                    EnumMember {
                        name: "Option2".to_string(),
                        value: EnumValue::String("option2".to_string()),
                    },
                ],
            }),
            required: true,
            unique: false,
            primary_key: false,
            default: None,
        }];

        // Test valid enum value
        let valid_json = r#"
        {
            "enum_col": "option1"
        }
        "#;

        let valid_result = serde_json::Deserializer::from_str(valid_json)
            .deserialize_any(&mut DataModelVisitor::new(&columns))
            .unwrap();

        let expected_valid = r#"{"enum_col":"option1"}"#;
        assert_eq!(
            String::from_utf8(valid_result),
            Ok(expected_valid.to_string())
        );

        // Test invalid enum value
        let invalid_json = r#"
        {
            "enum_col": "invalid_option"
        }
        "#;

        let invalid_result = serde_json::Deserializer::from_str(invalid_json)
            .deserialize_any(&mut DataModelVisitor::new(&columns));

        assert!(invalid_result.is_err());
        assert!(invalid_result
            .unwrap_err()
            .to_string()
            .contains("Invalid enum value at enum_col: invalid_option"));
    }

    #[test]
    fn test_nested() {
        let nested_columns = vec![
            Column {
                name: "nested_string".to_string(),
                data_type: ColumnType::String,
                required: true,
                unique: false,
                primary_key: false,
                default: None,
            },
            Column {
                name: "nested_int".to_string(),
                data_type: ColumnType::Int,
                required: false,
                unique: false,
                primary_key: false,
                default: None,
            },
        ];

        let columns = vec![
            Column {
                name: "top_level_string".to_string(),
                data_type: ColumnType::String,
                required: true,
                unique: false,
                primary_key: false,
                default: None,
            },
            Column {
                name: "nested_object".to_string(),
                data_type: ColumnType::Nested(Nested {
                    name: "nested".to_string(),
                    columns: nested_columns,
                }),
                required: true,
                unique: false,
                primary_key: false,
                default: None,
            },
        ];

        // Test valid nested object
        let valid_json = r#"
        {
            "top_level_string": "hello",
            "nested_object": {
                "nested_string": "world",
                "nested_int": 42
            }
        }
        "#;

        let valid_result = serde_json::Deserializer::from_str(valid_json)
            .deserialize_any(&mut DataModelVisitor::new(&columns))
            .unwrap();

        let expected_valid = r#"{"top_level_string":"hello","nested_object":{"nested_string":"world","nested_int":42}}"#;
        assert_eq!(
            String::from_utf8(valid_result),
            Ok(expected_valid.to_string())
        );

        // Test invalid nested object (missing required field)
        let invalid_json = r#"
        {
            "top_level_string": "hello",
            "nested_object": {
                "nested_int": 42
            }
        }
        "#;

        let invalid_result = serde_json::Deserializer::from_str(invalid_json)
            .deserialize_any(&mut DataModelVisitor::new(&columns));

        println!("{:?}", invalid_result);
        assert!(invalid_result.is_err());
        assert!(invalid_result
            .unwrap_err()
            .to_string()
            .contains("Missing fields: nested_object.nested_string"));
    }

    #[test]
    fn test_missing_non_required_field() {
        let columns = vec![
            Column {
                name: "required_field".to_string(),
                data_type: ColumnType::String,
                required: true,
                unique: false,
                primary_key: false,
                default: None,
            },
            Column {
                name: "optional_field".to_string(),
                data_type: ColumnType::Int,
                required: false,
                unique: false,
                primary_key: false,
                default: None,
            },
        ];

        let json = r#"
        {
            "required_field": "hello",
            "optional_field": null
        }
        "#;

        let result = serde_json::Deserializer::from_str(json)
            .deserialize_any(&mut DataModelVisitor::new(&columns))
            .unwrap();

        let expected = r#"{"required_field":"hello","optional_field":null}"#;
        assert_eq!(String::from_utf8(result), Ok(expected.to_string()));
    }
}
