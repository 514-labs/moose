use std::num::TryFromIntError;
use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, Days, NaiveDate};
use clickhouse_rs::errors::{Error, FromSqlError};
use clickhouse_rs::types::{ColumnType, Row};
use clickhouse_rs::types::{FromSql, FromSqlResult, Options, ValueRef};
use clickhouse_rs::ClientHandle;
use futures::stream::BoxStream;
use futures::StreamExt;
use serde::Serialize;
use serde::__private::from_utf8_lossy;
use serde_json::{json, Map, Value};
use swc_common::pass::Either;

use crate::framework::data_model::schema::EnumValue;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::olap::clickhouse::model::{ClickHouseColumnType, ClickHouseTable};

pub fn get_pool(click_house_config: &ClickHouseConfig) -> clickhouse_rs::Pool {
    let address = format!(
        "tcp://{}:{}",
        click_house_config.host, click_house_config.native_port
    );

    clickhouse_rs::Pool::new(
        Options::from_str(&address)
            .unwrap()
            .secure(click_house_config.use_ssl)
            .connection_timeout(Duration::from_secs(20))
            .database(&click_house_config.db_name)
            .username(&click_house_config.user)
            .password(&click_house_config.password),
    )
}

struct ValueRefWrapper<'a>(ValueRef<'a>);
impl<'a> FromSql<'a> for ValueRefWrapper<'a> {
    fn from_sql(value: ValueRef<'a>) -> FromSqlResult<ValueRefWrapper<'a>> {
        Ok(ValueRefWrapper(value))
    }
}

fn value_to_json(value_ref: &ValueRef, enum_mapping: &Option<Vec<&str>>) -> Result<Value, Error> {
    let result = match value_ref {
        ValueRef::Bool(v) => json!(v),
        ValueRef::UInt8(v) => json!(v),
        ValueRef::UInt16(v) => json!(v),
        ValueRef::UInt32(v) => json!(v),
        ValueRef::UInt64(v) => json!(v),
        ValueRef::Int8(v) => json!(v),
        ValueRef::Int16(v) => json!(v),
        ValueRef::Int32(v) => json!(v),
        ValueRef::Int64(v) => json!(v),
        // TODO: base64 encode if type is Bytes (probably Uint8Array in TS)
        // In clickhouse the String type means arbitrary bytes
        ValueRef::String(v) => json!(from_utf8_lossy(v)),
        ValueRef::Float32(v) => json!(v),
        ValueRef::Float64(v) => json!(v),
        ValueRef::Date(v) => {
            let unix_epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
            let naive_date = unix_epoch
                .checked_add_days(Days::new((*v).into()))
                .ok_or(Error::FromSql(FromSqlError::OutOfRange))?;
            json!(naive_date.to_string())
        }

        // in the following two cases the timezones are dropped
        ValueRef::DateTime(t, _tz) => {
            json!(DateTime::from_timestamp((*t).into(), 0)
                .ok_or(Error::FromSql(FromSqlError::OutOfRange))?
                .to_rfc3339())
        }
        ValueRef::DateTime64(value, (precision, _tz)) => {
            // See to_datetime_opt in clickhouse-rs
            let base10: i64 = 10;

            let nano = if *precision < 19 {
                value * base10.pow(9 - precision)
            } else {
                0_i64
            };

            let sec = nano / 1_000_000_000;
            let nsec: u32 = (nano - sec * 1_000_000_000).try_into().unwrap(); // always in range

            json!(DateTime::from_timestamp(sec, nsec)
                .ok_or(Error::FromSql(FromSqlError::OutOfRange))?)
        }

        ValueRef::Nullable(Either::Left(_)) => Value::Null,
        ValueRef::Nullable(Either::Right(v)) => value_to_json(v.as_ref(), enum_mapping)?,
        ValueRef::Array(_t, values) => json!(values
            .iter()
            .map(|v| value_to_json(v, enum_mapping))
            .collect::<Result<Vec<_>, Error>>()?),
        ValueRef::Decimal(d) => json!(f64::from(d.clone())), // consider using arbitrary_precision in serde_json
        ValueRef::Uuid(_) => json!(value_ref.to_string()),
        ValueRef::Enum16(_mapping, i) => convert_enum(i.internal(), enum_mapping),
        ValueRef::Enum8(_mapping, i) => convert_enum(i.internal(), enum_mapping),
        ValueRef::Ipv4(_) => todo!(),
        ValueRef::Ipv6(_) => todo!(),
        ValueRef::Map(_, _, _) => todo!(),
    };
    Ok(result)
}

fn convert_enum<I>(i: I, enum_mapping: &Option<Vec<&str>>) -> Value
where
    I: Serialize,
    usize: TryFrom<I, Error = TryFromIntError>,
{
    match enum_mapping {
        None => json!(i),
        // if they cannot be converted to usize but have an enum_mapping, the invariant -
        // enum_mapping is Some only when the TS enum has string values -
        // is broken
        Some(values) => json!(values[usize::try_from(i).unwrap() - 1]),
    }
}

/// enum_mappings[i] is None if the column is not an enum, or
/// is a TS enum that has integer values.
/// In other words, it's Some only when it is an enum that has string values
///
/// If the enum has int values, the JSON representation will be integers as well, so no need to map.
fn row_to_json<C>(row: &Row<'_, C>, enum_mappings: &[Option<Vec<&str>>]) -> Result<Value, Error>
where
    C: ColumnType,
{
    // can we use visitors to construct the JSON string directly,
    // without constructing the Value::Object first
    let mut result = Map::with_capacity(row.len());

    for (i, enum_mapping) in enum_mappings.iter().enumerate() {
        let value = value_to_json(&row.get::<ValueRefWrapper, _>(i).unwrap().0, enum_mapping);
        result.insert(row.name(i)?.into(), value?);
    }
    Ok(Value::Object(result))
}

fn column_type_to_enum_mapping(t: &ClickHouseColumnType) -> Option<Vec<&str>> {
    match t {
        ClickHouseColumnType::String
        | ClickHouseColumnType::Boolean
        | ClickHouseColumnType::ClickhouseInt(_)
        | ClickHouseColumnType::ClickhouseFloat(_)
        | ClickHouseColumnType::Decimal
        | ClickHouseColumnType::DateTime
        | ClickHouseColumnType::Json
        | ClickHouseColumnType::Bytes => None,
        ClickHouseColumnType::Array(t) => column_type_to_enum_mapping(t.as_ref()),
        ClickHouseColumnType::Enum(values) => values.values.first().and_then(|m| match m.value {
            EnumValue::Int(_) => None,
            EnumValue::String(_) => Some(
                values
                    .values
                    .iter()
                    .map(|member| match &member.value {
                        EnumValue::Int(_) => panic!("Mixed enum values."),
                        EnumValue::String(s) => s.as_str(),
                    })
                    .collect::<Vec<_>>(),
            ),
        }),
    }
}

pub async fn select_all_as_json<'a>(
    table: &'a ClickHouseTable,
    client: &'a mut ClientHandle,
    offset: i64,
) -> Result<BoxStream<'a, Result<Value, Error>>, Error> {
    let enum_mapping: Vec<Option<Vec<&str>>> = table
        .columns
        .iter()
        .map(|c| column_type_to_enum_mapping(&c.column_type))
        .collect();

    let key_columns = table
        .columns
        .iter()
        .filter_map(|c| {
            if c.primary_key {
                Some(c.name.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let order_by = if key_columns.is_empty() {
        "".to_string()
    } else {
        format!("ORDER BY {}", key_columns.join(", "))
    };
    let stream = client
        .query(&format!(
            "select * from {}.{} {} offset {}",
            table.db_name, table.name, order_by, offset
        ))
        .stream()
        .map(move |row| row_to_json(&row?, &enum_mapping));
    Ok(Box::pin(stream))
}
