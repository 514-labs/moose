/// # ClickHouse Alternative Client Module
///
/// This module provides an alternative client implementation for interacting with ClickHouse.
/// It focuses on JSON serialization of query results and table structure inspection.
///
/// The module includes functionality for:
/// - Converting ClickHouse data types to JSON
/// - Querying tables and returning results as JSON
/// - Inspecting table structures
/// - Converting between ClickHouse and standard column types
///
/// This client is used primarily for data exploration and infrastructure validation.
use std::collections::HashMap;
use std::num::TryFromIntError;
use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, Days, NaiveDate};
use clickhouse_rs::errors::FromSqlError;
use clickhouse_rs::types::{ColumnType, Row};
use clickhouse_rs::types::{FromSql, FromSqlResult, Options, ValueRef};
use clickhouse_rs::ClientHandle;
use futures::stream::BoxStream;
use futures::StreamExt;
use itertools::{Either, Itertools};
use log::{info, warn};
use serde::Serialize;
use serde::__private::from_utf8_lossy;
use serde_json::{json, Map, Value};

use crate::framework::core::infrastructure::table::{Column, EnumValue, Table};
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::olap::clickhouse::model::{
    ClickHouseColumn, ClickHouseColumnType, ClickHouseTable,
};

/// Creates a ClickHouse connection pool with the provided configuration.
///
/// # Arguments
/// * `click_house_config` - ClickHouse configuration
///
/// # Returns
/// * `clickhouse_rs::Pool` - Connection pool for ClickHouse
pub fn get_pool(click_house_config: &ClickHouseConfig) -> clickhouse_rs::Pool {
    let address = format!(
        "tcp://{}:{}",
        click_house_config.host, click_house_config.native_port
    );

    if click_house_config.use_ssl && click_house_config.native_port == 9000 {
        warn!(
            "The default secure native port is 9440 instead of 9000. You may get a timeout error."
        )
    }

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

/// Wrapper for ValueRef to implement FromSql trait.
struct ValueRefWrapper<'a>(ValueRef<'a>);
impl<'a> FromSql<'a> for ValueRefWrapper<'a> {
    fn from_sql(value: ValueRef<'a>) -> FromSqlResult<ValueRefWrapper<'a>> {
        Ok(ValueRefWrapper(value))
    }
}

/// Converts a ClickHouse ValueRef to a JSON Value.
///
/// This function handles all ClickHouse data types and converts them to appropriate
/// JSON representations. It also handles enum mappings for string enums.
///
/// # Arguments
/// * `value_ref` - ClickHouse value reference
/// * `enum_mapping` - Optional mapping for enum values
///
/// # Returns
/// * `Result<Value, clickhouse_rs::errors::Error>` - JSON value or error
fn value_to_json(
    value_ref: &ValueRef,
    enum_mapping: &Option<Vec<&str>>,
) -> Result<Value, clickhouse_rs::errors::Error> {
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
            let naive_date = unix_epoch.checked_add_days(Days::new((*v).into())).ok_or(
                clickhouse_rs::errors::Error::FromSql(FromSqlError::OutOfRange),
            )?;
            json!(naive_date.to_string())
        }

        // in the following two cases the timezones are dropped
        ValueRef::DateTime(t, _tz) => {
            json!(DateTime::from_timestamp((*t).into(), 0)
                .ok_or(clickhouse_rs::errors::Error::FromSql(
                    FromSqlError::OutOfRange
                ))?
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

            json!(DateTime::from_timestamp(sec, nsec).ok_or(
                clickhouse_rs::errors::Error::FromSql(FromSqlError::OutOfRange)
            )?)
        }

        ValueRef::Nullable(Either::Left(_)) => Value::Null,
        ValueRef::Nullable(Either::Right(v)) => value_to_json(v.as_ref(), enum_mapping)?,
        ValueRef::Array(_t, values) => json!(values
            .iter()
            .map(|v| value_to_json(v, enum_mapping))
            .collect::<Result<Vec<_>, clickhouse_rs::errors::Error>>()?),
        ValueRef::Decimal(d) => json!(f64::from(d.clone())), // consider using arbitrary_precision in serde_json
        ValueRef::Uuid(_) => json!(value_ref.to_string()),
        ValueRef::Enum16(_mapping, i) => convert_enum(i.internal(), enum_mapping),
        ValueRef::Enum8(_mapping, i) => convert_enum(i.internal(), enum_mapping),
        ValueRef::Ipv4(ip) => {
            let ip_str = format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
            json!(ip_str)
        }
        ValueRef::Ipv6(ip) => {
            json!(ip
                .chunks(2)
                .map(|chunk| { format!("{:02x}{:02x}", chunk[0], chunk[1]) })
                .join(":"))
        }
        ValueRef::Map(_, _, m) => Value::Object(
            m.iter()
                .map(|(k, v)| {
                    Ok::<_, clickhouse_rs::errors::Error>((
                        k.to_string(),
                        value_to_json(v, enum_mapping)?,
                    ))
                })
                .collect::<Result<_, _>>()?,
        ),
    };
    Ok(result)
}

/// Converts an enum value to a JSON value.
///
/// This function handles both integer and string enums. For string enums,
/// it uses the provided mapping to convert the integer value to a string.
///
/// # Arguments
/// * `i` - Enum integer value
/// * `enum_mapping` - Optional mapping for enum values
///
/// # Returns
/// * `Value` - JSON value for the enum
fn convert_enum<I>(i: I, enum_mapping: &Option<Vec<&str>>) -> Value
where
    I: Serialize,
    usize: TryFrom<I, Error = TryFromIntError>,
{
    match enum_mapping {
        None => json!(i),
        // unwrap is safe because of the invariant -
        // enum_mapping is Some only when the TS enum has string values
        Some(values) => json!(values[usize::try_from(i).unwrap() - 1]),
    }
}

/// Converts a ClickHouse row to a JSON object.
///
/// This function converts each column in the row to a JSON value and
/// combines them into a JSON object.
///
/// # Arguments
/// * `row` - ClickHouse row
/// * `enum_mappings` - Enum mappings for each column
///
/// # Returns
/// * `Result<Value, clickhouse_rs::errors::Error>` - JSON object or error
fn row_to_json<C>(
    row: &Row<'_, C>,
    enum_mappings: &[Option<Vec<&str>>],
) -> Result<Value, clickhouse_rs::errors::Error>
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

/// Converts a ClickHouse column type to an enum mapping.
///
/// This function extracts the enum mapping from a ClickHouse column type
/// if it's an enum with string values.
///
/// # Arguments
/// * `t` - ClickHouse column type
///
/// # Returns
/// * `Option<Vec<&str>>` - Enum mapping or None
fn column_type_to_enum_mapping(t: &ClickHouseColumnType) -> Option<Vec<&str>> {
    match t {
        ClickHouseColumnType::String
        | ClickHouseColumnType::Boolean
        | ClickHouseColumnType::ClickhouseInt(_)
        | ClickHouseColumnType::ClickhouseFloat(_)
        | ClickHouseColumnType::Decimal { .. }
        | ClickHouseColumnType::DateTime
        | ClickHouseColumnType::Date32
        | ClickHouseColumnType::Date
        | ClickHouseColumnType::Map(_, _)
        | ClickHouseColumnType::DateTime64 { .. }
        | ClickHouseColumnType::IpV4
        | ClickHouseColumnType::IpV6
        | ClickHouseColumnType::Json
        | ClickHouseColumnType::Uuid
        | ClickHouseColumnType::AggregateFunction { .. }
        | ClickHouseColumnType::Bytes => None,
        ClickHouseColumnType::Array(t) => column_type_to_enum_mapping(t.as_ref()),
        ClickHouseColumnType::NamedTuple(_) | ClickHouseColumnType::Nested(_) => {
            // Not entire sure I understand what this method does... do we just ignore the nested type?
            todo!("Implement the nested type mapper")
        }
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
        ClickHouseColumnType::Nullable(inner) => column_type_to_enum_mapping(inner),
        ClickHouseColumnType::LowCardinality(inner) => column_type_to_enum_mapping(inner),
    }
}

/// Executes a SELECT query and returns the results as a stream of JSON objects.
///
/// # Arguments
/// * `db_name` - Database name
/// * `table` - Table to query
/// * `client` - ClickHouse client
/// * `limit_offset_clause` - LIMIT/OFFSET clause for the query
///
/// # Returns
/// * `Result<BoxStream<'a, Result<Value, clickhouse_rs::errors::Error>>, clickhouse_rs::errors::Error>` - Stream of JSON objects or error
async fn select_as_json<'a>(
    db_name: &str,
    table: &'a ClickHouseTable,
    client: &'a mut ClientHandle,
    limit_offset_clause: &str,
) -> Result<BoxStream<'a, Result<Value, clickhouse_rs::errors::Error>>, clickhouse_rs::errors::Error>
{
    let enum_mapping: Vec<Option<Vec<&str>>> = table
        .columns
        .iter()
        .map(|c| column_type_to_enum_mapping(&c.column_type))
        .collect();

    let order_by = if !table.order_by.is_empty() {
        // Use explicit order_by fields if they exist
        format!("ORDER BY {}", table.order_by.join(", "))
    } else {
        // Fall back to primary key columns only if no explicit order_by is specified
        let key_columns = table.primary_key_columns();

        if key_columns.is_empty() {
            "".to_string()
        } else {
            format!("ORDER BY {}", key_columns.join(", "))
        }
    };

    let query = &format!(
        "select * from \"{}\".\"{}\" {} {}",
        db_name, table.name, order_by, limit_offset_clause
    );
    info!("select_as_json query: {}", query);
    let stream = client
        .query(query)
        .stream()
        .map(move |row| row_to_json(&row?, &enum_mapping));
    info!("select_as_json got data load stream.");
    Ok(Box::pin(stream))
}

/// Executes a SELECT query with an OFFSET clause and returns the results as a stream of JSON objects.
///
/// # Arguments
/// * `db_name` - Database name
/// * `table` - Table to query
/// * `client` - ClickHouse client
/// * `offset` - Offset for the query
///
/// # Returns
/// * `Result<BoxStream<'a, Result<Value, clickhouse_rs::errors::Error>>, clickhouse_rs::errors::Error>` - Stream of JSON objects or error
pub async fn select_all_as_json<'a>(
    db_name: &str,
    table: &'a ClickHouseTable,
    client: &'a mut ClientHandle,
    offset: i64,
) -> Result<BoxStream<'a, Result<Value, clickhouse_rs::errors::Error>>, clickhouse_rs::errors::Error>
{
    select_as_json(db_name, table, client, &format!("offset {offset}")).await
}

/// Executes a SELECT query with a LIMIT clause and returns the results as a stream of JSON objects.
///
/// # Arguments
/// * `db_name` - Database name
/// * `table` - Table to query
/// * `client` - ClickHouse client
/// * `limit` - Limit for the query
///
/// # Returns
/// * `Result<BoxStream<'a, Result<Value, clickhouse_rs::errors::Error>>, clickhouse_rs::errors::Error>` - Stream of JSON objects or error
pub async fn select_some_as_json<'a>(
    db_name: &str,
    table: &'a ClickHouseTable,
    client: &'a mut ClientHandle,
    limit: i64,
) -> Result<BoxStream<'a, Result<Value, clickhouse_rs::errors::Error>>, clickhouse_rs::errors::Error>
{
    select_as_json(db_name, table, client, &format!("limit {limit}")).await
}

/// Checks the structure of tables in the database and compares them with the expected structure.
///
/// This function queries the system.columns table to get information about the columns
/// in the specified database, then compares them with the expected structure.
///
/// # Arguments
/// * `client` - ClickHouse client
/// * `db_name` - Database name
/// * `tables` - Expected table structures
///
/// # Returns
/// * `Result<HashMap<String, Table>, clickhouse_rs::errors::Error>` - Actual table structures or error
pub async fn check_table(
    client: &mut ClientHandle,
    db_name: &str,
    tables: &HashMap<String, Table>,
) -> Result<HashMap<String, Table>, clickhouse_rs::errors::Error> {
    let columns_query = format!(
        r#"
        SELECT
            table,
            name,
            type,
            is_in_primary_key
        FROM system.columns 
        WHERE database = '{db_name}'
        "#
    );

    let block = client.query(&columns_query).fetch_all().await?;

    let mut table_columns = HashMap::<String, HashMap<String, ClickHouseColumnType>>::new();

    fn add_to_nested(
        existing_columns: &mut Vec<ClickHouseColumn>,
        inner_name: &str,
        t: ClickHouseColumnType,
    ) {
        match inner_name.split_once('.') {
            None => {
                let (column_type, required) = if let ClickHouseColumnType::Nullable(inner) = t {
                    (*inner, false)
                } else {
                    (t, true)
                };
                existing_columns.push(ClickHouseColumn {
                    name: inner_name.to_string(),
                    column_type,
                    required,
                    unique: false,
                    primary_key: false,
                    default: None,
                })
            }
            Some((nested, nested_inner)) => {
                let existing_nested = match existing_columns.iter_mut().find(|c| c.name == nested) {
                    None => {
                        existing_columns.push(ClickHouseColumn {
                            name: nested.to_string(),
                            column_type: ClickHouseColumnType::Nested(vec![]),
                            required: true,
                            unique: false,
                            primary_key: false,
                            default: None,
                        });
                        existing_columns.last_mut().unwrap()
                    }
                    Some(nested_column) => nested_column,
                };
                if let ClickHouseColumnType::Nested(v) = &mut existing_nested.column_type {
                    add_to_nested(v, nested_inner, t);
                } else {
                    unreachable!()
                }
            }
        }
    }

    for row in block.rows() {
        let table: String = row.get("table")?;

        let name: String = row.get("name")?;
        let type_str: String = row.get("type")?;

        if let Some(t) = ClickHouseColumnType::from_type_str(&type_str) {
            let columns = table_columns.entry(table).or_default();

            match name.split_once('.') {
                None => {
                    columns.insert(name, t);
                }
                Some((nested, nested_inner)) => {
                    let nested = columns
                        .entry(nested.to_string())
                        .or_insert(ClickHouseColumnType::Nested(vec![]));
                    if let ClickHouseColumnType::Nested(v) = nested {
                        add_to_nested(v, nested_inner, t)
                    }
                }
            }
        }
    }

    let mut existing_tables = tables.clone();

    for table in tables.values() {
        let mut db_columns = match table_columns.remove(&table.name) {
            None => {
                existing_tables.remove(&table.id());
                continue;
            }
            Some(columns) => columns,
        };
        let existing_table = existing_tables.get_mut(&table.id()).unwrap();
        existing_table
            .columns
            .retain_mut(|column| match db_columns.remove(&column.name) {
                None => false,
                Some(t) => {
                    let (data_type, required) = t.to_std_column_type();
                    column.required = required;
                    column.data_type = data_type;
                    true
                }
            });
        db_columns.into_iter().for_each(|(col_name, t)| {
            let (data_type, required) = t.to_std_column_type();
            existing_table.columns.push(Column {
                name: col_name,
                data_type,
                required,
                unique: false,
                primary_key: false,
                default: None,
                annotations: Default::default(),
            })
        })
    }

    Ok(existing_tables)
}
