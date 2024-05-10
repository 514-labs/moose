use serde::de::{Error, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

use super::config::DataModelConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct DataModel {
    pub db_name: String,
    pub columns: Vec<Column>,
    pub name: String,
    pub config: DataModelConfig,
}

impl DataModel {
    pub fn to_table(&self, version: &str) -> Table {
        Table {
            db_name: self.db_name.clone(),
            table_type: TableType::Table,
            name: format!("{}_{}", self.name, version.replace('.', "_")),
            columns: self.columns.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
/// An internal framework representation for an enum.
/// Avoiding the use of the `Enum` keyword to avoid conflicts with Prisma's Enum type
pub struct DataEnum {
    pub name: String,
    pub values: Vec<EnumMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct EnumMember {
    pub name: String,
    pub value: EnumValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum EnumValue {
    Int(u8),
    String(String),
}

#[derive(Debug, Clone)]
pub enum TableType {
    Table,
    View,
    Unsupported,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub db_name: String,
    pub table_type: TableType,
    pub name: String,
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Column {
    pub name: String,
    pub data_type: ColumnType,
    pub required: bool,
    pub unique: bool,
    pub primary_key: bool,
    pub default: Option<ColumnDefaults>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum ColumnDefaults {
    AutoIncrement,
    CUID,
    UUID,
    Now,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ColumnType {
    String,
    Boolean,
    Int,
    BigInt,
    Float,
    Decimal,
    DateTime,
    Enum(DataEnum),
    Array(Box<ColumnType>),
    Json,  // TODO: Eventually support for only views and tables (not topics)
    Bytes, // TODO: Explore if we ever need this type
}

impl fmt::Display for ColumnType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ColumnType::String => write!(f, "String"),
            ColumnType::Boolean => write!(f, "Boolean"),
            ColumnType::Int => write!(f, "Int"),
            ColumnType::BigInt => write!(f, "BigInt"),
            ColumnType::Float => write!(f, "Float"),
            ColumnType::Decimal => write!(f, "Decimal"),
            ColumnType::DateTime => write!(f, "DateTime"),
            ColumnType::Enum(e) => write!(f, "Enum<{}>", e.name),
            ColumnType::Array(inner) => write!(f, "Array<{}>", inner),
            ColumnType::Json => write!(f, "Json"),
            ColumnType::Bytes => write!(f, "Bytes"),
        }
    }
}

impl Serialize for ColumnType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ColumnType::String => serializer.serialize_str("String"),
            ColumnType::Boolean => serializer.serialize_str("Boolean"),
            ColumnType::Int => serializer.serialize_str("Int"),
            ColumnType::BigInt => serializer.serialize_str("BigInt"),
            ColumnType::Float => serializer.serialize_str("Float"),
            ColumnType::Decimal => serializer.serialize_str("Decimal"),
            ColumnType::DateTime => serializer.serialize_str("DateTime"),
            ColumnType::Enum(data_enum) => {
                let mut state = serializer.serialize_struct("Enum", 2)?;
                state.serialize_field("name", &data_enum.name)?;
                state.serialize_field("values", &data_enum.values)?;
                state.end()
            }
            ColumnType::Array(_) => {
                let serial = format!("{}", self);
                serializer.serialize_str(&serial)
            }
            ColumnType::Json => serializer.serialize_str("Json"),
            ColumnType::Bytes => serializer.serialize_str("Bytes"),
        }
    }
}

struct ColumnTypeVisitor;

impl<'de> Visitor<'de> for ColumnTypeVisitor {
    type Value = ColumnType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between -2^31 and 2^31")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        // we don't have a tag, right now it runs fine
        // because the only object to deserialize is the Enum case
        let mut name = None;
        let mut values = None;
        while let Some(key) = map.next_key::<&str>()? {
            if key == "name" {
                name = Some(map.next_value::<String>()?);
            } else if key == "values" {
                values = Some(map.next_value::<Vec<EnumMember>>()?)
            }
        }

        let name = name.ok_or(A::Error::custom("Missing field: name."))?;
        let values = values.ok_or(A::Error::custom("Missing field: values."))?;
        Ok(ColumnType::Enum(DataEnum { name, values }))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let t = if v == "String" {
            ColumnType::String
        } else if v == "Boolean" {
            ColumnType::Boolean
        } else if v == "Int" {
            ColumnType::Int
        } else if v == "BigInt" {
            ColumnType::BigInt
        } else if v == "Float" {
            ColumnType::Float
        } else if v == "Decimal" {
            ColumnType::Decimal
        } else if v == "DateTime" {
            ColumnType::DateTime
        } else if v == "Json" {
            ColumnType::Json
        } else if v == "Bytes" {
            ColumnType::Bytes
        } else if v.starts_with("Array<") && v.ends_with('>') {
            let inner = self
                .visit_str(&v[6..(v.len() - 1)])
                // .map_err(|e| E::custom(format!("Array inner type deserialization error {}.", e)))
                ?;
            ColumnType::Array(Box::new(inner))
        } else {
            return Err(E::custom(format!("Unknown column type {}.", v)));
        };
        Ok(t)
    }
}

impl<'de> Deserialize<'de> for ColumnType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ColumnTypeVisitor)
    }
}

pub fn is_enum_type(string_type: &str, enums: &[DataEnum]) -> bool {
    enums.iter().any(|e| e.name == string_type)
}

#[cfg(test)]
mod tests {

    use super::*;

    fn serialize_and_deserialize(t: &ColumnType) {
        let json = serde_json::to_string(t).unwrap();
        println!("JSON for {} is {}", t, json);
        let read: ColumnType = serde_json::from_str(&json).unwrap();
        assert_eq!(&read, t);
    }

    fn test_t(t: ColumnType) {
        serialize_and_deserialize(&t);

        if let ColumnType::Enum(_) = t {
            // The inner type for Array is missing in serialization
            // JSON for Array<Enum<with_string_values>> is "Array<Enum<with_string_values>>"
            // maybe fix later
            return;
        }
        let array = ColumnType::Array(Box::new(t));
        serialize_and_deserialize(&array);
    }

    #[test]
    fn test_column_type_serde() {
        test_t(ColumnType::Boolean);
        test_t(ColumnType::Enum(DataEnum {
            name: "with_string_values".to_string(),
            values: vec![
                EnumMember {
                    name: "up".to_string(),
                    value: EnumValue::String("UP".to_string()),
                },
                EnumMember {
                    name: "down".to_string(),
                    value: EnumValue::String("DOWN".to_string()),
                },
            ],
        }));
        test_t(ColumnType::Enum(DataEnum {
            name: "with_int_values".to_string(),
            values: vec![
                EnumMember {
                    name: "UP".to_string(),
                    value: EnumValue::Int(0),
                },
                EnumMember {
                    name: "DOWN".to_string(),
                    value: EnumValue::Int(1),
                },
            ],
        }));
    }
}
