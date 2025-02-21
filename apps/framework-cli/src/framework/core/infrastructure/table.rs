use protobuf::{EnumOrUnknown, MessageField};
use serde::de::{Error, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

use crate::framework::core::infrastructure_map::PrimitiveSignature;
use crate::framework::versions::Version;
use crate::proto::infrastructure_map::column_type;
use crate::proto::infrastructure_map::ColumnType as ProtoColumnType;
use crate::proto::infrastructure_map::Table as ProtoTable;
use crate::proto::infrastructure_map::{ColumnDefaults as ProtoColumnDefaults, SimpleColumnType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub order_by: Vec<String>,
    #[serde(default)]
    pub deduplicate: bool,

    pub version: Version,
    pub source_primitive: PrimitiveSignature,
}

impl Table {
    // This is only to be used in the context of the new core
    // currently name includes the version, here we are separating that out.
    pub fn id(&self) -> String {
        format!("{}_{}", self.name, self.version.as_suffix())
    }

    pub fn expanded_display(&self) -> String {
        format!(
            "Table: {} Version {} - {} - {} - deduplicate: {}",
            self.name,
            self.version,
            self.columns
                .iter()
                .map(|c| format!("{}: {}", c.name, c.data_type))
                .collect::<Vec<String>>()
                .join(", "),
            self.order_by.join(","),
            self.deduplicate
        )
    }

    pub fn short_display(&self) -> String {
        format!("Table: {} Version {}", self.name, self.version)
    }

    pub fn to_proto(&self) -> ProtoTable {
        ProtoTable {
            name: self.name.clone(),
            columns: self.columns.iter().map(|c| c.to_proto()).collect(),
            order_by: self.order_by.clone(),
            version: self.version.to_string(),
            source_primitive: MessageField::some(self.source_primitive.to_proto()),
            special_fields: Default::default(),
        }
    }

    pub fn from_proto(proto: ProtoTable) -> Self {
        Table {
            name: proto.name,
            columns: proto.columns.into_iter().map(Column::from_proto).collect(),
            order_by: proto.order_by,
            version: Version::from_string(proto.version),
            source_primitive: PrimitiveSignature::from_proto(proto.source_primitive.unwrap()),
            deduplicate: false, // TODO: Add to proto
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Column {
    pub name: String,
    pub data_type: ColumnType,
    pub required: bool,
    pub unique: bool,
    pub primary_key: bool,
    pub default: Option<ColumnDefaults>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum ColumnDefaults {
    AutoIncrement,
    CUID,
    UUID,
    Now,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ColumnType {
    String,
    Boolean,
    Int,
    BigInt,
    Float,
    Decimal,
    DateTime,
    Enum(DataEnum),
    Array {
        element_type: Box<ColumnType>,
        element_nullable: bool,
    },
    Nested(Nested),
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
            ColumnType::Array {
                element_type: inner,
                element_nullable: _,
            } => write!(f, "Array<{}>", inner),
            ColumnType::Nested(n) => write!(f, "Nested<{}>", n.name),
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
            ColumnType::Array {
                element_type,
                element_nullable,
            } => {
                let mut state = serializer.serialize_struct("Array", 1)?;
                state.serialize_field("elementType", element_type)?;
                state.serialize_field("elementNullable", element_nullable)?;
                state.end()
            }
            ColumnType::Nested(nested) => {
                let mut state = serializer.serialize_struct("Nested", 2)?;
                state.serialize_field("name", &nested.name)?;
                state.serialize_field("columns", &nested.columns)?;
                state.serialize_field("jwt", &nested.jwt)?;
                state.end()
            }
            ColumnType::Json => serializer.serialize_str("Json"),
            ColumnType::Bytes => serializer.serialize_str("Bytes"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
/// An internal framework representation for an enum.
/// Avoiding the use of the `Enum` keyword to avoid conflicts with Prisma's Enum type
pub struct DataEnum {
    pub name: String,
    pub values: Vec<EnumMember>,
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq, Hash)]
pub struct Nested {
    pub name: String,
    pub columns: Vec<Column>,
    #[serde(default)]
    pub jwt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct EnumMember {
    pub name: String,
    pub value: EnumValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum EnumValue {
    Int(u8),
    String(String),
}

struct ColumnTypeVisitor;

impl<'de> Visitor<'de> for ColumnTypeVisitor {
    type Value = ColumnType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string or an object for Enum/Array/Nested")
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
        } else {
            return Err(E::custom(format!("Unknown column type {}.", v)));
        };
        Ok(t)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut name = None;
        let mut values = None;
        let mut columns = None;
        let mut jwt = None;

        let mut element_type = None;
        let mut element_nullable = None;
        while let Some(key) = map.next_key::<String>()? {
            if key == "elementType" || key == "element_type" {
                element_type = Some(map.next_value::<ColumnType>().map_err(|e| {
                    A::Error::custom(format!("Array inner type deserialization error {}.", e))
                })?)
            } else if key == "elementNullable" || key == "element_nullable" {
                element_nullable = Some(map.next_value::<bool>()?)
            } else if key == "name" {
                name = Some(map.next_value::<String>()?);
            } else if key == "values" {
                values = Some(map.next_value::<Vec<EnumMember>>()?)
            } else if key == "columns" {
                columns = Some(map.next_value::<Vec<Column>>()?)
            } else if key == "jwt" {
                jwt = Some(map.next_value::<bool>()?)
            }
        }

        if let Some(element_type) = element_type {
            return Ok(ColumnType::Array {
                element_type: Box::new(element_type),
                element_nullable: element_nullable.unwrap_or(false),
            });
        }

        let name = name.ok_or(A::Error::custom("Missing field: name."))?;

        // we should probably add a tag to distinguish the object types
        // because we can distinguish them from the field names
        match (values, columns) {
            (None, None) => Err(A::Error::custom("Missing field: values/columns.")),
            (Some(values), _) => Ok(ColumnType::Enum(DataEnum { name, values })),
            (_, Some(columns)) => Ok(ColumnType::Nested(Nested {
                name,
                columns,
                jwt: jwt.unwrap_or(false),
            })),
        }
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

impl Column {
    pub fn to_proto(&self) -> crate::proto::infrastructure_map::Column {
        crate::proto::infrastructure_map::Column {
            name: self.name.clone(),
            data_type: MessageField::some(self.data_type.to_proto()),
            required: self.required,
            unique: self.unique,
            primary_key: self.primary_key,
            default: EnumOrUnknown::new(match &self.default {
                None => ProtoColumnDefaults::NONE,
                Some(column_default) => column_default.to_proto(),
            }),
            special_fields: Default::default(),
        }
    }

    pub fn from_proto(proto: crate::proto::infrastructure_map::Column) -> Self {
        Column {
            name: proto.name,
            data_type: ColumnType::from_proto(proto.data_type.unwrap()),
            required: proto.required,
            unique: proto.unique,
            primary_key: proto.primary_key,
            default: match proto
                .default
                .enum_value()
                .expect("Invalid default enum value")
            {
                ProtoColumnDefaults::NONE => None,
                default => Some(ColumnDefaults::from_proto(default)),
            },
        }
    }
}

impl ColumnType {
    pub fn to_proto(&self) -> ProtoColumnType {
        let t = match self {
            ColumnType::String => column_type::T::Simple(SimpleColumnType::STRING.into()),
            ColumnType::Boolean => column_type::T::Simple(SimpleColumnType::BOOLEAN.into()),
            ColumnType::Int => column_type::T::Simple(SimpleColumnType::INT.into()),
            ColumnType::BigInt => column_type::T::Simple(SimpleColumnType::BIGINT.into()),
            ColumnType::Float => column_type::T::Simple(SimpleColumnType::FLOAT.into()),
            ColumnType::Decimal => column_type::T::Simple(SimpleColumnType::DECIMAL.into()),
            ColumnType::DateTime => column_type::T::Simple(SimpleColumnType::DATETIME.into()),
            ColumnType::Enum(data_enum) => column_type::T::Enum(data_enum.to_proto()),
            ColumnType::Array {
                element_type,
                element_nullable: false,
            } => column_type::T::Array(Box::new(element_type.to_proto())),
            ColumnType::Array {
                element_type,
                element_nullable: true,
            } => column_type::T::ArrayOfNullable(Box::new(element_type.to_proto())),
            ColumnType::Nested(nested) => column_type::T::Nested(nested.to_proto()),
            ColumnType::Json => column_type::T::Simple(SimpleColumnType::JSON_COLUMN.into()),
            ColumnType::Bytes => column_type::T::Simple(SimpleColumnType::BYTES.into()),
        };
        ProtoColumnType {
            t: Some(t),
            special_fields: Default::default(),
        }
    }

    pub fn from_proto(proto: ProtoColumnType) -> Self {
        match proto.t.unwrap() {
            column_type::T::Simple(simple) => {
                match simple.enum_value().expect("Invalid simple type") {
                    SimpleColumnType::STRING => ColumnType::String,
                    SimpleColumnType::BOOLEAN => ColumnType::Boolean,
                    SimpleColumnType::INT => ColumnType::Int,
                    SimpleColumnType::BIGINT => ColumnType::BigInt,
                    SimpleColumnType::FLOAT => ColumnType::Float,
                    SimpleColumnType::DECIMAL => ColumnType::Decimal,
                    SimpleColumnType::DATETIME => ColumnType::DateTime,
                    SimpleColumnType::JSON_COLUMN => ColumnType::Json,
                    SimpleColumnType::BYTES => ColumnType::Bytes,
                }
            }
            column_type::T::Enum(data_enum) => ColumnType::Enum(DataEnum::from_proto(data_enum)),
            column_type::T::Array(element_type) => ColumnType::Array {
                element_type: Box::new(ColumnType::from_proto(*element_type)),
                element_nullable: false,
            },
            column_type::T::ArrayOfNullable(element_type) => ColumnType::Array {
                element_type: Box::new(ColumnType::from_proto(*element_type)),
                element_nullable: true,
            },
            column_type::T::Nested(nested) => ColumnType::Nested(Nested::from_proto(nested)),
        }
    }
}

impl DataEnum {
    pub fn to_proto(&self) -> crate::proto::infrastructure_map::DataEnum {
        crate::proto::infrastructure_map::DataEnum {
            name: self.name.clone(),
            values: self.values.iter().map(|v| v.to_proto()).collect(),
            special_fields: Default::default(),
        }
    }

    pub fn from_proto(proto: crate::proto::infrastructure_map::DataEnum) -> Self {
        DataEnum {
            name: proto.name,
            values: proto
                .values
                .into_iter()
                .map(EnumMember::from_proto)
                .collect(),
        }
    }
}

impl Nested {
    pub fn to_proto(&self) -> crate::proto::infrastructure_map::Nested {
        crate::proto::infrastructure_map::Nested {
            name: self.name.clone(),
            columns: self.columns.iter().map(|c| c.to_proto()).collect(),
            jwt: self.jwt,
            special_fields: Default::default(),
        }
    }

    pub fn from_proto(proto: crate::proto::infrastructure_map::Nested) -> Self {
        Nested {
            name: proto.name,
            columns: proto.columns.into_iter().map(Column::from_proto).collect(),
            jwt: proto.jwt,
        }
    }
}

impl EnumMember {
    pub fn to_proto(&self) -> crate::proto::infrastructure_map::EnumMember {
        crate::proto::infrastructure_map::EnumMember {
            name: self.name.clone(),
            value: MessageField::some(self.value.to_proto()),
            special_fields: Default::default(),
        }
    }

    pub fn from_proto(proto: crate::proto::infrastructure_map::EnumMember) -> Self {
        EnumMember {
            name: proto.name,
            value: EnumValue::from_proto(proto.value.unwrap()),
        }
    }
}

impl EnumValue {
    pub fn to_proto(&self) -> crate::proto::infrastructure_map::EnumValue {
        let value = match self {
            EnumValue::Int(i) => {
                crate::proto::infrastructure_map::enum_value::Value::IntValue(*i as i32)
            }
            EnumValue::String(s) => {
                crate::proto::infrastructure_map::enum_value::Value::StringValue(s.clone())
            }
        };
        crate::proto::infrastructure_map::EnumValue {
            value: Some(value),
            special_fields: Default::default(),
        }
    }

    pub fn from_proto(proto: crate::proto::infrastructure_map::EnumValue) -> Self {
        match proto.value.unwrap() {
            crate::proto::infrastructure_map::enum_value::Value::IntValue(i) => {
                EnumValue::Int(i as u8)
            }
            crate::proto::infrastructure_map::enum_value::Value::StringValue(s) => {
                EnumValue::String(s)
            }
        }
    }
}

impl ColumnDefaults {
    fn to_proto(&self) -> ProtoColumnDefaults {
        match self {
            ColumnDefaults::AutoIncrement => ProtoColumnDefaults::AUTO_INCREMENT,
            ColumnDefaults::CUID => ProtoColumnDefaults::CUID,
            ColumnDefaults::UUID => ProtoColumnDefaults::UUID,
            ColumnDefaults::Now => ProtoColumnDefaults::NOW,
        }
    }

    pub fn from_proto(proto: ProtoColumnDefaults) -> Self {
        match proto {
            ProtoColumnDefaults::AUTO_INCREMENT => ColumnDefaults::AutoIncrement,
            ProtoColumnDefaults::CUID => ColumnDefaults::CUID,
            ProtoColumnDefaults::UUID => ColumnDefaults::UUID,
            ProtoColumnDefaults::NOW => ColumnDefaults::Now,
            ProtoColumnDefaults::NONE => panic!("NONE should be handled as Option::None"),
        }
    }
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

        let array = ColumnType::Array {
            element_type: Box::new(t),
            element_nullable: false,
        };
        serialize_and_deserialize(&array);
        let nested_array = ColumnType::Array {
            element_type: Box::new(array),
            element_nullable: false,
        };
        serialize_and_deserialize(&nested_array);
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

    #[test]
    fn test_column_with_nested_type() {
        let nested_column = Column {
            name: "nested_column".to_string(),
            data_type: ColumnType::Nested(Nested {
                name: "nested".to_string(),
                columns: vec![],
                jwt: true,
            }),
            required: true,
            unique: false,
            primary_key: false,
            default: None,
        };

        let json = serde_json::to_string(&nested_column).unwrap();
        println!("Serialized JSON: {}", json);
        let deserialized: Column = serde_json::from_str(&json).unwrap();
        assert_eq!(nested_column, deserialized);
    }
}
