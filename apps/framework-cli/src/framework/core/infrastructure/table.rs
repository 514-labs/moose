use crate::framework::core::infrastructure_map::PrimitiveSignature;
use crate::framework::versions::Version;
use crate::proto::infrastructure_map;
use crate::proto::infrastructure_map::column_type::T;
use crate::proto::infrastructure_map::Decimal as ProtoDecimal;
use crate::proto::infrastructure_map::FloatType as ProtoFloatType;
use crate::proto::infrastructure_map::IntType as ProtoIntType;
use crate::proto::infrastructure_map::Table as ProtoTable;
use crate::proto::infrastructure_map::{column_type, DateType};
use crate::proto::infrastructure_map::{ColumnDefaults as ProtoColumnDefaults, SimpleColumnType};
use crate::proto::infrastructure_map::{ColumnType as ProtoColumnType, Map, Tuple};
use num_traits::ToPrimitive;
use protobuf::well_known_types::wrappers::StringValue;
use protobuf::{EnumOrUnknown, MessageField};
use serde::de::{Error, IgnoredAny, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::fmt;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Metadata {
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub order_by: Vec<String>,
    #[serde(default)]
    pub deduplicate: bool,
    #[serde(default)]
    pub engine: Option<String>,
    pub version: Option<Version>,
    pub source_primitive: PrimitiveSignature,
    pub metadata: Option<Metadata>,
}

impl Table {
    // This is only to be used in the context of the new core
    // currently name includes the version, here we are separating that out.
    pub fn id(&self) -> String {
        self.version.as_ref().map_or(self.name.clone(), |v| {
            format!("{}_{}", self.name, v.as_suffix())
        })
    }

    pub fn matches(&self, target_table_name: &str, target_table_version: Option<&Version>) -> bool {
        match target_table_version {
            None => self.name == target_table_name,
            Some(target_v) => {
                let expected_name = format!("{}_{}", target_table_name, target_v.as_suffix());
                self.name == expected_name
            }
        }
    }

    pub fn expanded_display(&self) -> String {
        format!(
            "Table: {} Version {:?} - {} - {} - deduplicate: {}",
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
        format!(
            "Table: {name} Version {version:?}",
            name = self.name,
            version = self.version
        )
    }

    /// Returns the names of all primary key columns in this table
    pub fn primary_key_columns(&self) -> Vec<&str> {
        self.columns
            .iter()
            .filter_map(|c| {
                if c.primary_key {
                    Some(c.name.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn to_proto(&self) -> ProtoTable {
        ProtoTable {
            name: self.name.clone(),
            columns: self.columns.iter().map(|c| c.to_proto()).collect(),
            order_by: self.order_by.clone(),
            version: self.version.as_ref().map(|v| v.to_string()),
            source_primitive: MessageField::some(self.source_primitive.to_proto()),
            deduplicate: self.deduplicate,
            engine: MessageField::from_option(self.engine.as_ref().map(|engine| StringValue {
                value: engine.to_string(),
                special_fields: Default::default(),
            })),
            metadata: MessageField::from_option(self.metadata.as_ref().map(|m| {
                infrastructure_map::Metadata {
                    description: m.description.clone().unwrap_or_default(),
                    special_fields: Default::default(),
                }
            })),
            special_fields: Default::default(),
        }
    }

    pub fn from_proto(proto: ProtoTable) -> Self {
        Table {
            name: proto.name,
            columns: proto.columns.into_iter().map(Column::from_proto).collect(),
            order_by: proto.order_by,
            version: proto.version.map(Version::from_string),
            source_primitive: PrimitiveSignature::from_proto(proto.source_primitive.unwrap()),
            deduplicate: proto.deduplicate,
            engine: proto.engine.into_option().map(|wrapper| wrapper.value),
            metadata: proto.metadata.into_option().map(|m| Metadata {
                description: if m.description.is_empty() {
                    None
                } else {
                    Some(m.description)
                },
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Column {
    pub name: String,
    pub data_type: ColumnType,
    // TODO: move `required: false` to `data_type: Nullable(...)`
    pub required: bool,
    pub unique: bool,
    pub primary_key: bool,
    pub default: Option<ColumnDefaults>,
    #[serde(default)]
    pub annotations: Vec<(String, Value)>, // workaround for needing to Hash
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum ColumnDefaults {
    AutoIncrement,
    CUID,
    UUID,
    Now,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum IntType {
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Int256,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    UInt256,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum FloatType {
    Float32,
    Float64,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ColumnType {
    String,
    Boolean,
    Int(IntType),
    BigInt,
    Float(FloatType),
    Decimal {
        precision: u8,
        scale: u8,
    },
    DateTime {
        precision: Option<u8>,
    },
    // most databases use 4 bytes or more for a date
    // in clickhouse that's `Date32`
    Date,
    // `Date` in clickhouse is 2 bytes
    Date16,
    Enum(DataEnum),
    Array {
        element_type: Box<ColumnType>,
        element_nullable: bool,
    },
    Nullable(Box<ColumnType>),
    NamedTuple(Vec<(String, ColumnType)>),
    Map {
        key_type: Box<ColumnType>,
        value_type: Box<ColumnType>,
    },
    Nested(Nested),
    Json,  // TODO: Eventually support for only views and tables (not topics)
    Bytes, // TODO: Explore if we ever need this type
    Uuid,
    IpV4,
    IpV6,
}

impl fmt::Display for ColumnType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ColumnType::String => write!(f, "String"),
            ColumnType::Boolean => write!(f, "Boolean"),
            ColumnType::Int(int_type) => int_type.fmt(f),
            ColumnType::BigInt => write!(f, "BigInt"),
            ColumnType::Float(float_type) => float_type.fmt(f),
            ColumnType::Decimal { precision, scale } => {
                write!(f, "Decimal({precision}, {scale})")
            }
            ColumnType::DateTime { precision: None } => write!(f, "DateTime"),
            ColumnType::DateTime {
                precision: Some(precision),
            } => write!(f, "DateTime({precision})"),
            ColumnType::Enum(e) => write!(f, "Enum<{}>", e.name),
            ColumnType::Array {
                element_type: inner,
                element_nullable: _,
            } => write!(f, "Array<{inner}>"),
            ColumnType::Nested(n) => write!(f, "Nested<{}>", n.name),
            ColumnType::Json => write!(f, "Json"),
            ColumnType::Bytes => write!(f, "Bytes"),
            ColumnType::Uuid => write!(f, "UUID"),
            ColumnType::Date => write!(f, "Date"),
            ColumnType::Date16 => write!(f, "Date16"),
            ColumnType::IpV4 => write!(f, "IPv4"),
            ColumnType::IpV6 => write!(f, "IPv6"),
            ColumnType::Nullable(inner) => write!(f, "Nullable<{inner}>"),
            ColumnType::NamedTuple(fields) => {
                write!(f, "NamedTuple<")?;
                fields
                    .iter()
                    .try_for_each(|(name, t)| write!(f, "{name}: {t}"))?;
                write!(f, ">")
            }
            ColumnType::Map {
                key_type,
                value_type,
            } => write!(f, "Map<{key_type}, {value_type}>"),
        }
    }
}

impl Serialize for ColumnType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ColumnType::String => serializer.serialize_str("String"),
            ColumnType::Boolean => serializer.serialize_str("Boolean"),
            ColumnType::Int(int_type) => serializer.serialize_str(&format!("{int_type:?}")),
            ColumnType::BigInt => serializer.serialize_str("BigInt"),
            ColumnType::Float(float_type) => serializer.serialize_str(&format!("{float_type:?}")),
            ColumnType::Decimal { precision, scale } => {
                serializer.serialize_str(&format!("Decimal({precision}, {scale})"))
            }
            ColumnType::DateTime { precision: None } => serializer.serialize_str("DateTime"),
            ColumnType::DateTime {
                precision: Some(precision),
            } => serializer.serialize_str(&format!("DateTime({precision})")),
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
                let mut state = serializer.serialize_struct("Array", 2)?;
                state.serialize_field("elementType", element_type)?;
                state.serialize_field("elementNullable", element_nullable)?;
                state.end()
            }
            ColumnType::Nested(nested) => {
                let mut state = serializer.serialize_struct("Nested", 3)?;
                state.serialize_field("name", &nested.name)?;
                state.serialize_field("columns", &nested.columns)?;
                state.serialize_field("jwt", &nested.jwt)?;
                state.end()
            }
            ColumnType::Json => serializer.serialize_str("Json"),
            ColumnType::Bytes => serializer.serialize_str("Bytes"),
            ColumnType::Uuid => serializer.serialize_str("UUID"),
            ColumnType::Date => serializer.serialize_str("Date"),
            ColumnType::Date16 => serializer.serialize_str("Date16"),
            ColumnType::IpV4 => serializer.serialize_str("IPv4"),
            ColumnType::IpV6 => serializer.serialize_str("IPv6"),
            ColumnType::NamedTuple(fields) => {
                let mut state = serializer.serialize_struct("NamedTuple", 1)?;
                state.serialize_field("fields", &fields)?;
                state.end()
            }
            ColumnType::Nullable(inner) => {
                let mut state = serializer.serialize_struct("Nullable", 1)?;
                state.serialize_field("nullable", inner)?;
                state.end()
            }
            ColumnType::Map {
                key_type,
                value_type,
            } => {
                let mut state = serializer.serialize_struct("Map", 2)?;
                state.serialize_field("keyType", key_type)?;
                state.serialize_field("valueType", value_type)?;
                state.end()
            }
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
            ColumnType::Int(IntType::Int64)
        } else if v == "Int8" {
            ColumnType::Int(IntType::Int8)
        } else if v == "Int16" {
            ColumnType::Int(IntType::Int16)
        } else if v == "Int32" {
            ColumnType::Int(IntType::Int32)
        } else if v == "Int64" {
            ColumnType::Int(IntType::Int64)
        } else if v == "Int128" {
            ColumnType::Int(IntType::Int128)
        } else if v == "Int256" {
            ColumnType::Int(IntType::Int256)
        } else if v == "UInt8" {
            ColumnType::Int(IntType::UInt8)
        } else if v == "UInt16" {
            ColumnType::Int(IntType::UInt16)
        } else if v == "UInt32" {
            ColumnType::Int(IntType::UInt32)
        } else if v == "UInt64" {
            ColumnType::Int(IntType::UInt64)
        } else if v == "UInt128" {
            ColumnType::Int(IntType::UInt128)
        } else if v == "UInt256" {
            ColumnType::Int(IntType::UInt256)
        } else if v == "BigInt" {
            ColumnType::BigInt
        } else if v == "Float" {
            // usually "float" means single precision, but backwards compatibility
            ColumnType::Float(FloatType::Float64)
        } else if v == "Float32" {
            ColumnType::Float(FloatType::Float32)
        } else if v == "Float64" {
            ColumnType::Float(FloatType::Float64)
        } else if v.starts_with("Decimal") {
            let mut precision = 10;
            let mut scale = 0;

            if v.starts_with("Decimal(") {
                let params = v
                    .trim_start_matches("Decimal(")
                    .trim_end_matches(')')
                    .split(',')
                    .map(|s| s.trim().parse::<u8>())
                    .collect::<Vec<_>>();

                if let Some(Ok(p)) = params.first() {
                    precision = *p;
                }
                if let Some(Ok(s)) = params.get(1) {
                    scale = *s;
                }
            }
            ColumnType::Decimal { precision, scale }
        } else if v == "DateTime" {
            ColumnType::DateTime { precision: None }
        } else if v.starts_with("DateTime(") {
            let precision = v
                .strip_prefix("DateTime(")
                .unwrap()
                .strip_suffix(")")
                .and_then(|p| p.trim().parse::<u8>().ok())
                .ok_or_else(|| E::custom(format!("Invalid DateTime precision: {v}")))?;
            ColumnType::DateTime {
                precision: Some(precision),
            }
        } else if v == "Date" {
            ColumnType::Date
        } else if v == "Date16" {
            ColumnType::Date16
        } else if v == "Json" {
            ColumnType::Json
        } else if v == "Bytes" {
            ColumnType::Bytes
        } else if v == "UUID" {
            ColumnType::Uuid
        } else if v == "IPv4" {
            ColumnType::IpV4
        } else if v == "IPv6" {
            ColumnType::IpV6
        } else {
            return Err(E::custom(format!("Unknown column type {v}.")));
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
        let mut fields = None;
        let mut jwt = None;
        let mut nullable_inner = None;

        let mut element_type = None;
        let mut element_nullable = None;
        let mut key_type = None;
        let mut value_type = None;
        while let Some(key) = map.next_key::<String>()? {
            if key == "elementType" || key == "element_type" {
                element_type = Some(map.next_value::<ColumnType>().map_err(|e| {
                    A::Error::custom(format!("Array inner type deserialization error {e}."))
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
            } else if key == "fields" {
                fields = Some(map.next_value::<Vec<(String, ColumnType)>>()?)
            } else if key == "nullable" {
                nullable_inner = Some(map.next_value::<ColumnType>()?)
            } else if key == "keyType" || key == "key_type" {
                key_type = Some(map.next_value::<ColumnType>().map_err(|e| {
                    A::Error::custom(format!("Map key type deserialization error {e}."))
                })?)
            } else if key == "valueType" || key == "value_type" {
                value_type = Some(map.next_value::<ColumnType>().map_err(|e| {
                    A::Error::custom(format!("Map value type deserialization error {e}."))
                })?)
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }
        if let Some(inner) = nullable_inner {
            return Ok(ColumnType::Nullable(Box::new(inner)));
        }

        if let Some(fields) = fields {
            return Ok(ColumnType::NamedTuple(fields));
        }

        if let Some(element_type) = element_type {
            return Ok(ColumnType::Array {
                element_type: Box::new(element_type),
                element_nullable: element_nullable.unwrap_or(false),
            });
        }

        if let Some(key_type) = key_type {
            if let Some(value_type) = value_type {
                return Ok(ColumnType::Map {
                    key_type: Box::new(key_type),
                    value_type: Box::new(value_type),
                });
            } else {
                return Err(A::Error::custom("Map type missing valueType field"));
            }
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
            annotations: self
                .annotations
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            special_fields: Default::default(),
        }
    }

    pub fn from_proto(proto: crate::proto::infrastructure_map::Column) -> Self {
        let mut annotations: Vec<(String, Value)> = proto
            .annotations
            .into_iter()
            .map(|(k, v)| (k, serde_json::from_str(&v).unwrap()))
            .collect();
        annotations.sort_by(|a, b| a.0.cmp(&b.0));

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
            annotations,
        }
    }
}

impl ColumnType {
    pub fn to_proto(&self) -> ProtoColumnType {
        let t = match self {
            ColumnType::String => column_type::T::Simple(SimpleColumnType::STRING.into()),
            ColumnType::Boolean => column_type::T::Simple(SimpleColumnType::BOOLEAN.into()),
            ColumnType::Int(int_type) => column_type::T::Int(
                (match int_type {
                    IntType::Int8 => ProtoIntType::INT8,
                    IntType::Int16 => ProtoIntType::INT16,
                    IntType::Int32 => ProtoIntType::INT32,
                    IntType::Int64 => ProtoIntType::INT64,
                    IntType::Int128 => ProtoIntType::INT128,
                    IntType::Int256 => ProtoIntType::INT256,
                    IntType::UInt8 => ProtoIntType::UINT8,
                    IntType::UInt16 => ProtoIntType::UINT16,
                    IntType::UInt32 => ProtoIntType::UINT32,
                    IntType::UInt64 => ProtoIntType::UINT64,
                    IntType::UInt128 => ProtoIntType::UINT128,
                    IntType::UInt256 => ProtoIntType::UINT256,
                })
                .into(),
            ),
            ColumnType::BigInt => column_type::T::Simple(SimpleColumnType::BIGINT.into()),
            ColumnType::Float(float_type) => column_type::T::Float(
                (match float_type {
                    FloatType::Float32 => ProtoFloatType::FLOAT32,
                    FloatType::Float64 => ProtoFloatType::FLOAT64,
                })
                .into(),
            ),
            ColumnType::Decimal { precision, scale } => column_type::T::Decimal(ProtoDecimal {
                precision: *precision as i32,
                scale: *scale as i32,
                special_fields: Default::default(),
            }),
            ColumnType::DateTime { precision: None } => {
                column_type::T::Simple(SimpleColumnType::DATETIME.into())
            }
            ColumnType::DateTime {
                precision: Some(precision),
            } => column_type::T::DateTime(DateType {
                precision: (*precision).into(),
                special_fields: Default::default(),
            }),
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
            ColumnType::Uuid => column_type::T::Simple(SimpleColumnType::UUID_TYPE.into()),
            ColumnType::Date => T::Simple(SimpleColumnType::DATE.into()),
            ColumnType::Date16 => T::Simple(SimpleColumnType::DATE16.into()),
            ColumnType::IpV4 => T::Simple(SimpleColumnType::IPV4.into()),
            ColumnType::IpV6 => T::Simple(SimpleColumnType::IPV6.into()),
            ColumnType::NamedTuple(fields) => T::Tuple(Tuple {
                names: fields.iter().map(|(name, _)| name.clone()).collect(),
                types: fields.iter().map(|(_, t)| t.to_proto()).collect(),
                special_fields: Default::default(),
            }),
            ColumnType::Nullable(inner) => column_type::T::Nullable(Box::new(inner.to_proto())),
            ColumnType::Map {
                key_type,
                value_type,
            } => column_type::T::Map(Map {
                key_type: MessageField::some(key_type.to_proto()),
                value_type: MessageField::some(value_type.to_proto()),
                special_fields: Default::default(),
            }),
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
                    SimpleColumnType::INT => ColumnType::Int(IntType::Int64),
                    SimpleColumnType::BIGINT => ColumnType::BigInt,
                    SimpleColumnType::FLOAT => ColumnType::Float(FloatType::Float64),
                    SimpleColumnType::DECIMAL => ColumnType::Decimal {
                        precision: 10,
                        scale: 0,
                    },
                    SimpleColumnType::DATETIME => ColumnType::DateTime { precision: None },
                    SimpleColumnType::JSON_COLUMN => ColumnType::Json,
                    SimpleColumnType::BYTES => ColumnType::Bytes,
                    SimpleColumnType::UUID_TYPE => ColumnType::Uuid,
                    SimpleColumnType::DATE => ColumnType::Date,
                    SimpleColumnType::DATE16 => ColumnType::Date16,
                    SimpleColumnType::IPV4 => ColumnType::IpV4,
                    SimpleColumnType::IPV6 => ColumnType::IpV6,
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
            T::Decimal(d) => ColumnType::Decimal {
                scale: d.scale.to_u8().unwrap(),
                precision: d.precision.to_u8().unwrap(),
            },
            T::Float(f) => ColumnType::Float(match f.enum_value_or(ProtoFloatType::FLOAT64) {
                ProtoFloatType::FLOAT64 => FloatType::Float64,
                ProtoFloatType::FLOAT32 => FloatType::Float32,
            }),
            T::Int(i) => ColumnType::Int(match i.enum_value_or(ProtoIntType::INT64) {
                ProtoIntType::INT64 => IntType::Int64,
                ProtoIntType::INT8 => IntType::Int8,
                ProtoIntType::INT16 => IntType::Int16,
                ProtoIntType::INT32 => IntType::Int32,
                ProtoIntType::INT128 => IntType::Int128,
                ProtoIntType::INT256 => IntType::Int256,
                ProtoIntType::UINT8 => IntType::UInt8,
                ProtoIntType::UINT16 => IntType::UInt16,
                ProtoIntType::UINT32 => IntType::UInt32,
                ProtoIntType::UINT64 => IntType::UInt64,
                ProtoIntType::UINT128 => IntType::UInt128,
                ProtoIntType::UINT256 => IntType::UInt256,
            }),
            T::DateTime(DateType { precision, .. }) => ColumnType::DateTime {
                precision: Some(precision.to_u8().unwrap()),
            },
            T::Tuple(t) if t.names.len() == t.types.len() => ColumnType::NamedTuple(
                t.names
                    .iter()
                    .zip(t.types.iter())
                    .map(|(name, t)| (name.clone(), Self::from_proto(t.clone())))
                    .collect(),
            ),
            T::Tuple(t) if t.names.is_empty() => {
                panic!("Unnamed tuples not supported yet.")
            }
            T::Tuple(_) => {
                panic!("Mismatched length between names and types.")
            }
            T::Nullable(inner) => ColumnType::Nullable(Box::new(Self::from_proto(*inner))),
            T::Map(map) => ColumnType::Map {
                key_type: Box::new(Self::from_proto(map.key_type.clone().unwrap())),
                value_type: Box::new(Self::from_proto(map.value_type.clone().unwrap())),
            },
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
        println!("JSON for {t} is {json}");
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
            annotations: vec![],
        };

        let json = serde_json::to_string(&nested_column).unwrap();
        println!("Serialized JSON: {json}");
        let deserialized: Column = serde_json::from_str(&json).unwrap();
        assert_eq!(nested_column, deserialized);
    }
}
