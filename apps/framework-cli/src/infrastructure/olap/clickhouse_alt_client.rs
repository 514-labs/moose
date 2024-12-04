use std::collections::{HashMap, HashSet};
use std::num::TryFromIntError;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, Days, NaiveDate};
use clickhouse_rs::errors::FromSqlError;
use clickhouse_rs::types::{ColumnType, Row};
use clickhouse_rs::types::{FromSql, FromSqlResult, Options, ValueRef};
use clickhouse_rs::ClientHandle;
use futures::stream::BoxStream;
use futures::StreamExt;
use itertools::Either;
use log::{info, warn};
use serde::__private::from_utf8_lossy;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::framework::core::code_loader::FrameworkObjectVersions;
use crate::framework::core::infrastructure::table::{Column, EnumValue, Table};
use crate::framework::core::infrastructure_map::InfrastructureMap;
use crate::framework::data_model::model::DataModel;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::olap::clickhouse::model::{
    ClickHouseColumn, ClickHouseColumnType, ClickHouseTable,
};

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

struct ValueRefWrapper<'a>(ValueRef<'a>);
impl<'a> FromSql<'a> for ValueRefWrapper<'a> {
    fn from_sql(value: ValueRef<'a>) -> FromSqlResult<ValueRefWrapper<'a>> {
        Ok(ValueRefWrapper(value))
    }
}

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
        // unwrap is safe because of the invariant -
        // enum_mapping is Some only when the TS enum has string values
        Some(values) => json!(values[usize::try_from(i).unwrap() - 1]),
    }
}

/// enum_mappings[i] is None if the column is not an enum, or
/// is a TS enum that has integer values.
/// In other words, it's Some only when it is an enum that has string values
///
/// If the enum has int values, the JSON representation will be integers as well, so no need to map.
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
        ClickHouseColumnType::Nested(_) => {
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
    }
}

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

pub async fn select_all_as_json<'a>(
    db_name: &str,
    table: &'a ClickHouseTable,
    client: &'a mut ClientHandle,
    offset: i64,
) -> Result<BoxStream<'a, Result<Value, clickhouse_rs::errors::Error>>, clickhouse_rs::errors::Error>
{
    select_as_json(db_name, table, client, &format!("offset {}", offset)).await
}

pub async fn select_some_as_json<'a>(
    db_name: &str,
    table: &'a ClickHouseTable,
    client: &'a mut ClientHandle,
    limit: i64,
) -> Result<BoxStream<'a, Result<Value, clickhouse_rs::errors::Error>>, clickhouse_rs::errors::Error>
{
    select_as_json(db_name, table, client, &format!("limit {}", limit)).await
}

async fn create_state_table(
    client: &mut ClientHandle,
    click_house_config: &ClickHouseConfig,
) -> Result<(), clickhouse_rs::errors::Error> {
    let sql = format!(
        r#"CREATE TABLE IF NOT EXISTS "{}"._MOOSE_STATE(
    timestamp DateTime('UTC') DEFAULT now(),
    state String
)ENGINE = MergeTree
PRIMARY KEY timestamp
ORDER BY timestamp"#,
        click_house_config.db_name
    );
    client.execute(sql).await
}

pub async fn store_current_state(
    client: &mut ClientHandle,
    framework_object_versions: &FrameworkObjectVersions,
    aggregations: &HashSet<String>,
    click_house_config: &ClickHouseConfig,
) -> Result<(), StateStorageError> {
    create_state_table(client, click_house_config).await?;

    let data = clickhouse_rs::Block::new().column(
        "state",
        vec![serde_json::to_string(&ApplicationState::from((
            framework_object_versions,
            aggregations,
        )))?],
    );
    client
        .insert(
            format!("\"{}\"._MOOSE_STATE", click_house_config.db_name),
            data,
        )
        .await?;
    Ok(())
}

pub async fn get_state(
    click_house_config: &ClickHouseConfig,
) -> Result<Option<ApplicationState>, StateStorageError> {
    let pool = get_pool(click_house_config);
    let mut client = pool.get_handle().await?;
    retrieve_current_state(&mut client, click_house_config).await
}

pub async fn retrieve_current_state(
    client: &mut ClientHandle,
    click_house_config: &ClickHouseConfig,
) -> Result<Option<ApplicationState>, StateStorageError> {
    create_state_table(client, click_house_config).await?;
    let block = client
        .query(format!(
            "SELECT state from \"{}\"._MOOSE_STATE ORDER BY timestamp DESC LIMIT 1",
            click_house_config.db_name
        ))
        .fetch_all()
        .await?;

    let state_str = block
        .rows()
        .map(|row| row.get(0).unwrap())
        .collect::<Vec<String>>();
    match state_str.len() {
        0 => Ok(None),
        1 => Ok(serde_json::from_str(state_str.first().unwrap())?),
        len => panic!("LIMIT 1 but got {} rows: {:?}", len, state_str),
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StateStorageError {
    #[error("Failed to (de)serialize the state")]
    SerdeError(#[from] serde_json::Error),
    #[error("Clickhouse error")]
    ClickhouseError(#[from] clickhouse_rs::errors::Error),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationState {
    pub models: Vec<(String, Vec<Model>)>,
    pub aggregations: HashSet<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub data_model: DataModel,
    pub original_file_path: PathBuf,
}

impl From<(&FrameworkObjectVersions, &HashSet<String>)> for ApplicationState {
    fn from((versions, aggregations): (&FrameworkObjectVersions, &HashSet<String>)) -> Self {
        let models = versions
            .previous_version_models
            .iter()
            .chain(std::iter::once((
                &versions.current_version,
                &versions.current_models,
            )))
            .map(|(version, schema_version)| {
                let models = schema_version
                    .models
                    .values()
                    .map(|model| Model {
                        data_model: model.data_model.clone(),
                        original_file_path: model.original_file_path.clone(),
                    })
                    .collect();
                (version.clone(), models)
            })
            .collect();
        ApplicationState {
            models,
            aggregations: aggregations.clone(),
        }
    }
}

pub async fn store_infrastructure_map(
    client: &mut ClientHandle,
    clickhouse_config: &ClickHouseConfig,
    infrastructure_map: &InfrastructureMap,
) -> Result<(), StateStorageError> {
    let data = clickhouse_rs::Block::new().column(
        "infra_map",
        vec![serde_json::to_string(infrastructure_map)?],
    );
    client
        .insert(
            format!("\"{}\"._MOOSE_STATE_V2", clickhouse_config.db_name),
            data,
        )
        .await?;
    Ok(())
}

async fn create_infrastructure_map_table(
    client: &mut ClientHandle,
    click_house_config: &ClickHouseConfig,
) -> Result<(), clickhouse_rs::errors::Error> {
    let sql = format!(
        r#"CREATE TABLE IF NOT EXISTS "{}"._MOOSE_STATE_V2 (
            timestamp DateTime('UTC') DEFAULT now(),
            infra_map String
        ) ENGINE = MergeTree
        PRIMARY KEY timestamp
        ORDER BY timestamp"#,
        click_house_config.db_name
    );
    client.execute(sql).await
}

pub async fn retrieve_infrastructure_map(
    client: &mut ClientHandle,
    click_house_config: &ClickHouseConfig,
) -> Result<Option<InfrastructureMap>, StateStorageError> {
    create_infrastructure_map_table(client, click_house_config).await?;

    let block = client
        .query(format!(
            "SELECT infra_map from \"{}\"._MOOSE_STATE_V2 ORDER BY timestamp DESC LIMIT 1",
            click_house_config.db_name
        ))
        .fetch_all()
        .await?;

    let inframap_string = block
        .rows()
        .map(|row| row.get(0).unwrap())
        .collect::<Vec<String>>();

    match inframap_string.len() {
        0 => Ok(None),
        1 => Ok(serde_json::from_str(inframap_string.first().unwrap())?),
        len => panic!("LIMIT 1 but got {} rows: {:?}", len, inframap_string),
    }
}

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
        WHERE database = '{}'
        "#,
        db_name
    );

    let block = client.query(&columns_query).fetch_all().await?;

    let mut table_columns = HashMap::<String, HashMap<String, (ClickHouseColumnType, bool)>>::new();

    fn add_to_nested(
        existing_columns: &mut Vec<ClickHouseColumn>,
        inner_name: &str,
        t: ClickHouseColumnType,
        required: bool,
    ) {
        match inner_name.split_once('.') {
            None => existing_columns.push(ClickHouseColumn {
                name: inner_name.to_string(),
                column_type: t,
                required,
                unique: false,
                primary_key: false,
                default: None,
            }),
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
                    add_to_nested(v, nested_inner, t, required);
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

        if let Some((t, required)) = ClickHouseColumnType::from_type_str(&type_str) {
            let columns = table_columns.entry(table).or_default();

            match name.split_once('.') {
                None => {
                    columns.insert(name, (t, required));
                }
                Some((nested, nested_inner)) => {
                    let adsf = columns
                        .entry(nested.to_string())
                        .or_insert((ClickHouseColumnType::Nested(vec![]), true));
                    if let (ClickHouseColumnType::Nested(v), true) = adsf {
                        add_to_nested(v, nested_inner, t, required)
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
                Some((t, required)) => {
                    column.required = required;

                    column.data_type = t.to_std_column_type();
                    true
                }
            });
        db_columns
            .into_iter()
            .for_each(|(col_name, (t, required))| {
                existing_table.columns.push(Column {
                    name: col_name,
                    data_type: t.to_std_column_type(),
                    required,
                    unique: false,
                    primary_key: false,
                    default: None,
                })
            })
    }

    Ok(existing_tables)
}
