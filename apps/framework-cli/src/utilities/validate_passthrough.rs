use crate::framework::core::infrastructure::table::{Column, ColumnType, DataEnum, EnumValue};
use crate::infrastructure::processes::kafka_clickhouse_sync::IPV4_PATTERN;
use itertools::Either;
use regex::Regex;
use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserializer, Serialize, Serializer};
use serde_json::Serializer as JsonSerializer;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Write};
use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::LazyLock;

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
impl<S> SerializeValue for DummyWrapper<'_, S>
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
        Err(E::custom(format!("Invalid enum value: {v}")))
    }
}

struct ParentContext<'a> {
    parent: Option<&'a ParentContext<'a>>,
    field_name: Either<&'a str, usize>,
}
impl ParentContext<'_> {
    fn bump_index(&mut self) {
        match self.field_name {
            Either::Left(_) => {
                panic!("Expecting array index case")
            }
            Either::Right(ref mut i) => {
                *i += 1;
            }
        }
    }

    fn get_path(&self) -> String {
        add_path_component(parent_context_to_string(self.parent), self.field_name)
    }
}

struct ValueVisitor<'a, S: SerializeValue> {
    t: &'a ColumnType,
    required: bool,
    write_to: &'a mut S,
    context: ParentContext<'a>,
    jwt_claims: Option<&'a Value>,
}
impl<'de, S: SerializeValue> DeserializeSeed<'de> for &mut ValueVisitor<'_, S> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}
impl<'de, S: SerializeValue> Visitor<'de> for &mut ValueVisitor<'_, S> {
    type Value = ();

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        match self.t {
            ColumnType::Boolean => formatter.write_str("a boolean value"),
            ColumnType::Int(_) => formatter.write_str("an integer value"),
            ColumnType::Float(_) => formatter.write_str("a floating-point value"),
            ColumnType::String => formatter.write_str("a string value"),
            ColumnType::DateTime { .. } => formatter.write_str("a datetime value"),
            ColumnType::Enum(_) => formatter.write_str("an enum value"),
            ColumnType::Array { .. } => formatter.write_str("an array value"),
            ColumnType::Nested(_) => formatter.write_str("a nested object"),

            ColumnType::BigInt
            | ColumnType::Date
            | ColumnType::Date16
            | ColumnType::IpV4
            | ColumnType::IpV6
            | ColumnType::Decimal { .. }
            | ColumnType::Json
            | ColumnType::Bytes => formatter.write_str("a value matching the column type"),
            ColumnType::Uuid => formatter.write_str("a UUID"),
            ColumnType::Nullable(inner) => {
                write!(formatter, "a nullable value of type {inner}")
            }
            ColumnType::NamedTuple(fields) => {
                write!(formatter, "an object with fields: ")?;
                for (i, (name, field_type)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(formatter, ", ")?;
                    }
                    write!(formatter, "{name}: {field_type}")?;
                }
                Ok(())
            }
            ColumnType::Map {
                key_type,
                value_type,
            } => {
                write!(
                    formatter,
                    "a map with key type {key_type} and value type {value_type}"
                )
            }
            ColumnType::Geo(geo_type) => {
                write!(formatter, "a geo value of type {geo_type}")
            }
        }?;
        write!(formatter, " at {}", self.get_path())
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
            ColumnType::Int(_) => self.write_to.serialize_value(&v).map_err(Error::custom),
            ColumnType::Float(_) => self.write_to.serialize_value(&v).map_err(Error::custom),
            ColumnType::Decimal { .. } => self.write_to.serialize_value(&v).map_err(Error::custom),
            ColumnType::Enum(enum_def) => handle_enum_value(self.write_to, enum_def, v),
            _ => Err(Error::invalid_type(serde::de::Unexpected::Signed(v), &self)),
        }
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.t {
            ColumnType::Int(_) => self.write_to.serialize_value(&v).map_err(Error::custom),
            ColumnType::Float(_) => self.write_to.serialize_value(&v).map_err(Error::custom),
            ColumnType::Decimal { .. } => self.write_to.serialize_value(&v).map_err(Error::custom),
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
            ColumnType::DateTime { .. } => {
                let seconds = v.trunc() as i64;
                let nanos = ((v.fract() * 1_000_000_000.0).round() as u32).min(999_999_999);
                let date = chrono::DateTime::from_timestamp(seconds, nanos)
                    .ok_or(E::custom("Invalid timestamp"))?;
                self.write_to
                    .serialize_value(&date.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
                    .map_err(Error::custom)
            }
            ColumnType::Decimal { .. } => self.write_to.serialize_value(&v).map_err(Error::custom),
            ColumnType::Float(_) => self.write_to.serialize_value(&v).map_err(Error::custom),
            _ => Err(Error::invalid_type(serde::de::Unexpected::Float(v), &self)),
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.t {
            ColumnType::String => self.write_to.serialize_value(v).map_err(Error::custom),
            ColumnType::DateTime { .. } => {
                chrono::DateTime::parse_from_rfc3339(v).map_err(|_| {
                    E::custom(format!("Invalid date format at {}", self.get_path()))
                })?;

                self.write_to.serialize_value(v).map_err(Error::custom)
            }
            ColumnType::IpV4 => {
                if IPV4_PATTERN.is_match(v) {
                    self.write_to.serialize_value(v).map_err(Error::custom)
                } else {
                    Err(E::custom(format!(
                        "Invalid IPv4 format at {}",
                        self.get_path()
                    )))
                }
            }
            ColumnType::IpV6 => {
                if std::net::Ipv6Addr::from_str(v).is_ok() {
                    self.write_to.serialize_value(v).map_err(Error::custom)
                } else {
                    Err(E::custom(format!(
                        "Invalid IPv6 format at {}",
                        self.get_path()
                    )))
                }
            }
            ColumnType::Decimal { .. } => {
                if DECIMAL_REGEX.is_match(v) {
                    self.write_to.serialize_value(v).map_err(Error::custom)
                } else {
                    Err(E::custom(format!(
                        "Invalid decimal format at {}",
                        self.get_path()
                    )))
                }
            }
            ColumnType::Date | ColumnType::Date16 => {
                if DATE_REGEX.is_match(v) {
                    self.write_to.serialize_value(v).map_err(Error::custom)
                } else {
                    Err(E::custom(format!(
                        "Invalid date format at {}",
                        self.get_path()
                    )))
                }
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
            ColumnType::Uuid if uuid::Uuid::parse_str(v).is_ok() => {
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
            ColumnType::Array {
                element_type: ref inner_type,
                element_nullable,
            } => {
                self.write_to
                    .serialize_value(&SeqAccessSerializer {
                        inner_type,
                        inner_required: *element_nullable,
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
                let inner = DataModelVisitor::with_context(
                    &fields.columns,
                    Some(&self.context),
                    self.jwt_claims,
                );
                let serializer = MapAccessSerializer {
                    inner: RefCell::new(inner),
                    map: RefCell::new(map),
                    _phantom_data: &PHANTOM_DATA,
                };
                self.write_to
                    .serialize_value(&serializer)
                    .map_err(A::Error::custom)
            }
            ColumnType::NamedTuple(ref fields) => {
                let columns: Vec<Column> = fields
                    .iter()
                    .map(|(name, t)| {
                        let (required, data_type) = match t {
                            ColumnType::Nullable(inner) => (false, inner.as_ref().clone()),
                            _ => (true, t.clone()),
                        };
                        Column {
                            name: name.clone(),
                            data_type,
                            required,
                            unique: false,
                            primary_key: false,
                            default: None,
                            annotations: vec![],
                            comment: None,
                        }
                    })
                    .collect();
                let inner =
                    DataModelVisitor::with_context(&columns, Some(&self.context), self.jwt_claims);
                let serializer = MapAccessSerializer {
                    inner: RefCell::new(inner),
                    map: RefCell::new(map),
                    _phantom_data: &PHANTOM_DATA,
                };
                self.write_to
                    .serialize_value(&serializer)
                    .map_err(A::Error::custom)
            }
            ColumnType::Map {
                key_type,
                value_type,
            } => {
                struct MapPassThrough<'de, 'a, MA: MapAccess<'de>> {
                    parent_context: &'a ParentContext<'a>,
                    key_type: &'a ColumnType,
                    value_type: &'a ColumnType,
                    map: RefCell<MA>,
                    _phantom_data: &'de PhantomData<()>,
                }
                impl<'de, MA: MapAccess<'de>> Serialize for MapPassThrough<'de, '_, MA> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: Serializer,
                    {
                        use serde::ser::Error;
                        let mut inner = serializer.serialize_map(None)?;

                        let mut map = self.map.borrow_mut();
                        while let Some(key) = map.next_key::<String>().map_err(S::Error::custom)? {
                            validate_map_key::<S::Error>(&key, self.key_type, self.parent_context)?;
                            inner.serialize_key(&key)?;
                            let mut value_visitor = ValueVisitor {
                                t: self.value_type,
                                required: true,
                                write_to: &mut inner,
                                context: ParentContext {
                                    parent: Some(self.parent_context),
                                    field_name: Either::Left(&key),
                                },
                                jwt_claims: None,
                            };
                            map.next_value_seed(&mut value_visitor)
                                .map_err(S::Error::custom)?;
                            value_visitor.context.field_name = Either::Left("");
                        }
                        inner.end().map_err(S::Error::custom)
                    }
                }
                let map = MapPassThrough {
                    parent_context: &self.context,
                    key_type,
                    value_type,
                    map: RefCell::new(map),
                    _phantom_data: &PhantomData,
                };
                self.write_to
                    .serialize_value(&map)
                    .map_err(A::Error::custom)
            }
            ColumnType::Json => {
                struct JsonPassThrough<'de, MA: MapAccess<'de>> {
                    map: RefCell<MA>,
                    _phantom_data: &'de PhantomData<()>,
                }
                impl<'de, MA: MapAccess<'de>> Serialize for JsonPassThrough<'de, MA> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: Serializer,
                    {
                        use serde::ser::Error;
                        let mut inner = serializer.serialize_map(None)?;
                        let mut map = self.map.borrow_mut();
                        while let Some(key) = map.next_key::<String>().map_err(S::Error::custom)? {
                            inner.serialize_key(&key).map_err(S::Error::custom)?;
                            SerializeMap::serialize_value(
                                &mut inner,
                                &map.next_value::<serde_json::Value>()
                                    .map_err(S::Error::custom)?,
                            )?;
                        }
                        inner.end().map_err(S::Error::custom)
                    }
                }
                self.write_to
                    .serialize_value(&JsonPassThrough {
                        map: RefCell::new(map),
                        _phantom_data: &PhantomData,
                    })
                    .map_err(A::Error::custom)
            }
            _ => Err(A::Error::invalid_type(serde::de::Unexpected::Map, &self)),
        }
    }
}

/// Validate a key string according to the given key type
fn validate_map_key<E>(
    key_str: &str,
    key_type: &ColumnType,
    context: &ParentContext,
) -> Result<(), E>
where
    E: serde::ser::Error,
{
    match key_type {
        ColumnType::String => Ok(()), // String keys are always valid
        ColumnType::Int(_) => {
            key_str.parse::<i64>().map_err(|_| {
                E::custom(format!(
                    "Invalid integer key '{}' for Map at {}",
                    key_str,
                    context.get_path()
                ))
            })?;
            Ok(())
        }
        ColumnType::Float(_) => {
            key_str.parse::<f64>().map_err(|_| {
                E::custom(format!(
                    "Invalid float key '{}' for Map at {}",
                    key_str,
                    context.get_path()
                ))
            })?;
            Ok(())
        }
        ColumnType::IpV4 => {
            if IPV4_PATTERN.is_match(key_str) {
                Ok(())
            } else {
                Err(E::custom(format!(
                    "Invalid IPv4 key '{}' for Map at {}",
                    key_str,
                    context.get_path()
                )))
            }
        }
        ColumnType::IpV6 => {
            if std::net::Ipv6Addr::from_str(key_str).is_ok() {
                Ok(())
            } else {
                Err(E::custom(format!(
                    "Invalid IPv6 key '{}' for Map at {}",
                    key_str,
                    context.get_path()
                )))
            }
        }
        ColumnType::Uuid => {
            uuid::Uuid::parse_str(key_str).map_err(|_| {
                E::custom(format!(
                    "Invalid UUID key '{}' for Map at {}",
                    key_str,
                    context.get_path()
                ))
            })?;
            Ok(())
        }
        _ => Err(E::custom(format!(
            "Unsupported key type {:?} for Map at {}",
            key_type,
            context.get_path()
        ))),
    }
}
impl<S: SerializeValue> ValueVisitor<'_, S> {
    fn get_path(&self) -> String {
        self.context.get_path()
    }
}

impl<'de, A: SeqAccess<'de>> Serialize for SeqAccessSerializer<'_, 'de, A> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut write_to = serializer.serialize_seq(None)?;
        let mut visitor = ValueVisitor {
            t: self.inner_type,
            required: self.inner_required,
            write_to: &mut DummyWrapper(&mut write_to),
            context: ParentContext {
                parent: Some(self.context),
                field_name: Either::Right(0),
            },
            jwt_claims: None,
        };
        let mut seq = self.seq.borrow_mut();
        while let Some(()) = seq
            .next_element_seed(&mut visitor)
            .map_err(serde::ser::Error::custom)?
        {
            visitor.context.bump_index();
        }
        write_to.end()
    }
}

static DATE_REGEX: LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"^[0-9]{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$").unwrap()
});
pub static DECIMAL_REGEX: LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^-?\d+(\.\d+)?$").unwrap());
static PHANTOM_DATA: PhantomData<()> = PhantomData {};
// RefCell for interior mutability
// generally serialization for T should not change the T
// but here we read elements/entries off from the SeqAccess/MapAccess
// as we put it into the output JSON
struct SeqAccessSerializer<'a, 'de, A: SeqAccess<'de>> {
    inner_type: &'a ColumnType,
    inner_required: bool,
    seq: RefCell<A>,
    _phantom_data: &'de PhantomData<()>,
    context: &'a ParentContext<'a>,
}
struct MapAccessSerializer<'de, 'a, A: MapAccess<'de>> {
    inner: RefCell<DataModelVisitor<'a>>,
    map: RefCell<A>,
    _phantom_data: &'de PhantomData<()>,
}

impl<'de, A: MapAccess<'de>> Serialize for MapAccessSerializer<'de, '_, A> {
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
    jwt_claims: Option<&'a Value>,
}
impl<'a> DataModelVisitor<'a> {
    pub fn new(columns: &[Column], jwt_claims: Option<&'a Value>) -> Self {
        Self::with_context(columns, None, jwt_claims)
    }

    fn with_context(
        columns: &[Column],
        parent_context: Option<&'a ParentContext<'a>>,
        jwt_claims: Option<&'a Value>,
    ) -> Self {
        DataModelVisitor {
            columns: columns
                .iter()
                .map(|c| (c.name.clone(), (c.clone(), State { seen: false })))
                .collect(),
            parent_context,
            jwt_claims,
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
                        field_name: Either::Left(&key),
                    },
                    jwt_claims: self.jwt_claims,
                };
                map.next_value_seed(&mut visitor)?;
            } else {
                map.next_value::<serde::de::IgnoredAny>()?;
            }
        }
        let mut missing_fields: Vec<String> = Vec::new();
        for (column, state) in self.columns.values_mut() {
            if !state.seen && column.required {
                let parent_path = parent_context_to_string(self.parent_context);
                let path = add_path_component(parent_path, Either::Left(&column.name));

                if is_nested_with_jwt(&column.data_type) {
                    if let Some(jwt_claims) = self.jwt_claims {
                        map_serializer
                            .serialize_key(&column.name)
                            .map_err(A::Error::custom)?;
                        map_serializer
                            .serialize_value(jwt_claims)
                            .map_err(A::Error::custom)?;
                    }
                } else {
                    missing_fields.push(path);
                }
            }
            state.seen = false
        }

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
            add_path_component(parent_context_to_string(*parent), *field_name)
        }
        None => String::new(),
    }
}

impl<'de> Visitor<'de> for &mut DataModelVisitor<'_> {
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
impl<'de> DeserializeSeed<'de> for &mut DataModelVisitor<'_> {
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
impl<'de> Visitor<'de> for &mut DataModelArrayVisitor<'_> {
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

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        Ok(vec![self.inner.visit_map(map)?])
    }
}

fn add_path_component(mut path: String, field_name: Either<&str, usize>) -> String {
    if !path.is_empty() {
        path.push('.');
    }
    match field_name {
        Either::Left(field_name) => {
            path.push_str(field_name);
        }
        Either::Right(index) => {
            write!(path, "{index}").unwrap();
        }
    }

    path
}

fn is_nested_with_jwt(column_type: &ColumnType) -> bool {
    match column_type {
        ColumnType::Nested(nested) => nested.jwt,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::core::infrastructure::table::{
        Column, ColumnType, DataEnum, EnumMember, EnumValue, FloatType, IntType, Nested,
    };

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
                annotations: vec![],
                comment: None,
            },
            Column {
                name: "int_col".to_string(),
                data_type: ColumnType::Int(IntType::Int64),
                required: true,
                unique: false,
                primary_key: false,
                default: None,
                annotations: vec![],
                comment: None,
            },
            Column {
                name: "float_col".to_string(),
                data_type: ColumnType::Float(FloatType::Float64),
                required: true,
                unique: false,
                primary_key: false,
                default: None,
                annotations: vec![],
                comment: None,
            },
            Column {
                name: "bool_col".to_string(),
                data_type: ColumnType::Boolean,
                required: true,
                unique: false,
                primary_key: false,
                default: None,
                annotations: vec![],
                comment: None,
            },
            Column {
                name: "date_col".to_string(),
                data_type: ColumnType::DateTime { precision: None },
                required: true,
                unique: false,
                primary_key: false,
                default: None,
                annotations: vec![],
                comment: None,
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
            .deserialize_any(&mut DataModelVisitor::new(&columns, None))
            .unwrap();

        let expected = r#"{"string_col":"test","int_col":42,"float_col":3.14,"bool_col":true,"date_col":"2024-09-10T17:34:51+00:00"}"#;

        assert_eq!(String::from_utf8(result), Ok(expected.to_string()));
    }

    #[test]
    fn test_bad_date_format() {
        let columns = vec![Column {
            name: "date_col".to_string(),
            data_type: ColumnType::DateTime { precision: None },
            required: true,
            unique: false,
            primary_key: false,
            default: None,
            annotations: vec![],
            comment: None,
        }];

        let json = r#"
        {
            "date_col": "2024-09-10"
        }
        "#;

        let result = serde_json::Deserializer::from_str(json)
            .deserialize_any(&mut DataModelVisitor::new(&columns, None));

        println!("{result:?}");
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
            data_type: ColumnType::Array {
                element_type: Box::new(ColumnType::Int(IntType::Int64)),
                element_nullable: false,
            },
            required: true,
            unique: false,
            primary_key: false,
            default: None,
            annotations: vec![],
            comment: None,
        }];

        let json = r#"
        {
            "array_col": [1, 2, 3, 4, 5]
        }
        "#;

        let result = serde_json::Deserializer::from_str(json)
            .deserialize_any(&mut DataModelVisitor::new(&columns, None))
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
            annotations: vec![],
            comment: None,
        }];

        // Test valid enum value
        let valid_json = r#"
        {
            "enum_col": "option1"
        }
        "#;

        let valid_result = serde_json::Deserializer::from_str(valid_json)
            .deserialize_any(&mut DataModelVisitor::new(&columns, None))
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
            .deserialize_any(&mut DataModelVisitor::new(&columns, None));

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
                annotations: vec![],
                comment: None,
            },
            Column {
                name: "nested_int".to_string(),
                data_type: ColumnType::Int(IntType::Int64),
                required: false,
                unique: false,
                primary_key: false,
                default: None,
                annotations: vec![],
                comment: None,
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
                annotations: vec![],
                comment: None,
            },
            Column {
                name: "nested_object".to_string(),
                data_type: ColumnType::Nested(Nested {
                    name: "nested".to_string(),
                    columns: nested_columns,
                    jwt: false,
                }),
                required: true,
                unique: false,
                primary_key: false,
                default: None,
                annotations: vec![],
                comment: None,
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
            .deserialize_any(&mut DataModelVisitor::new(&columns, None))
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
            .deserialize_any(&mut DataModelVisitor::new(&columns, None));

        println!("{invalid_result:?}");
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
                annotations: vec![],
                comment: None,
            },
            Column {
                name: "optional_field".to_string(),
                data_type: ColumnType::Int(IntType::Int64),
                required: false,
                unique: false,
                primary_key: false,
                default: None,
                annotations: vec![],
                comment: None,
            },
        ];

        let json = r#"
        {
            "required_field": "hello",
            "optional_field": null
        }
        "#;

        let result = serde_json::Deserializer::from_str(json)
            .deserialize_any(&mut DataModelVisitor::new(&columns, None))
            .unwrap();

        let expected = r#"{"required_field":"hello","optional_field":null}"#;
        assert_eq!(String::from_utf8(result), Ok(expected.to_string()));
    }

    #[test]
    fn test_jwt() {
        let nested_columns = vec![
            Column {
                name: "iss".to_string(),
                data_type: ColumnType::String,
                required: true,
                unique: false,
                primary_key: false,
                default: None,
                annotations: vec![],
                comment: None,
            },
            Column {
                name: "aud".to_string(),
                data_type: ColumnType::String,
                required: true,
                unique: false,
                primary_key: false,
                default: None,
                annotations: vec![],
                comment: None,
            },
            Column {
                name: "exp".to_string(),
                data_type: ColumnType::Float(FloatType::Float64),
                required: true,
                unique: false,
                primary_key: false,
                default: None,
                annotations: vec![],
                comment: None,
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
                annotations: vec![],
                comment: None,
            },
            Column {
                name: "jwt_object".to_string(),
                data_type: ColumnType::Nested(Nested {
                    name: "nested".to_string(),
                    columns: nested_columns,
                    jwt: true,
                }),
                required: true,
                unique: false,
                primary_key: false,
                default: None,
                annotations: vec![],
                comment: None,
            },
        ];

        // JWT purposely missing in the request
        let valid_json = r#"
        {
            "top_level_string": "hello"
        }
        "#;

        // Fake JWT claims to pass to the visitor
        let jwt_claims = serde_json::json!({
            "iss": "issuer",
            "aud": "audience",
            "exp": 2043418466
        });

        let valid_result = serde_json::Deserializer::from_str(valid_json)
            .deserialize_any(&mut DataModelVisitor::new(&columns, Some(&jwt_claims)))
            .unwrap();

        // Visitor should've injected the jwt claims
        let expected_valid = format!(r#"{{"top_level_string":"hello","jwt_object":{jwt_claims}}}"#);

        assert_eq!(
            String::from_utf8(valid_result),
            Ok(expected_valid.to_string())
        );
    }

    #[test]
    fn test_map_validation() {
        let columns = vec![Column {
            name: "user_scores".to_string(),
            data_type: ColumnType::Map {
                key_type: Box::new(ColumnType::String),
                value_type: Box::new(ColumnType::Int(IntType::Int64)),
            },
            required: true,
            unique: false,
            primary_key: false,
            default: None,
            annotations: vec![],
            comment: None,
        }];

        // Test valid map
        let valid_json = r#"
        {
            "user_scores": {
                "alice": 100,
                "bob": 85,
                "charlie": 92
            }
        }
        "#;

        let valid_result = serde_json::Deserializer::from_str(valid_json)
            .deserialize_any(&mut DataModelVisitor::new(&columns, None))
            .unwrap();

        let expected_valid = r#"{"user_scores":{"alice":100,"bob":85,"charlie":92}}"#;
        assert_eq!(
            String::from_utf8(valid_result),
            Ok(expected_valid.to_string())
        );

        // Test invalid map (wrong value type)
        let invalid_json = r#"
        {
            "user_scores": {
                "alice": "not_a_number",
                "bob": 85
            }
        }
        "#;

        let invalid_result = serde_json::Deserializer::from_str(invalid_json)
            .deserialize_any(&mut DataModelVisitor::new(&columns, None));

        assert!(invalid_result.is_err());
        let error = invalid_result.unwrap_err();
        println!("{error}");
        assert!(error.to_string().contains(r#"invalid type: string "not_a_number", expected an integer value at user_scores.alice at line"#));
    }

    #[test]
    fn test_map_with_numeric_keys() {
        let columns = vec![Column {
            name: "id_to_name".to_string(),
            data_type: ColumnType::Map {
                key_type: Box::new(ColumnType::Int(IntType::Int64)),
                value_type: Box::new(ColumnType::String),
            },
            required: true,
            unique: false,
            primary_key: false,
            default: None,
            annotations: vec![],
            comment: None,
        }];

        // Test valid map with numeric keys (as strings in JSON)
        let valid_json = r#"
        {
            "id_to_name": {
                "123": "Alice",
                "456": "Bob"
            }
        }
        "#;

        let valid_result = serde_json::Deserializer::from_str(valid_json)
            .deserialize_any(&mut DataModelVisitor::new(&columns, None))
            .unwrap();

        let expected_valid = r#"{"id_to_name":{"123":"Alice","456":"Bob"}}"#;
        assert_eq!(
            String::from_utf8(valid_result),
            Ok(expected_valid.to_string())
        );

        // Test invalid map with non-numeric keys
        let invalid_json = r#"
        {
            "id_to_name": {
                "not_a_number": "Alice",
                "456": "Bob"
            }
        }
        "#;

        let invalid_result = serde_json::Deserializer::from_str(invalid_json)
            .deserialize_any(&mut DataModelVisitor::new(&columns, None));

        assert!(invalid_result.is_err());
        assert!(invalid_result
            .unwrap_err()
            .to_string()
            .contains("Invalid integer key"));
    }
}
