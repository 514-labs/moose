use crate::framework::core::code_loader::FrameworkObject;
use crate::framework::core::infrastructure::table::ColumnType;
use anyhow::bail;
use itertools::Itertools;
use serde::__private::from_utf8_lossy;
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;

pub async fn import_csv_file(
    data_model: &FrameworkObject,
    file: &Path,
    destination: &str,
) -> anyhow::Result<()> {
    let mut rdr = csv::Reader::from_path(file)?;

    let headers: Vec<String> = rdr.headers()?.into_iter().map(|s| s.to_string()).collect();

    let client = reqwest::Client::new();

    let types: HashMap<String, ColumnType> = data_model
        .data_model
        .columns
        .iter()
        .map(|col| (col.name.clone(), col.data_type.clone()))
        .collect();

    for chunks in &rdr.records().chunks(1024) {
        let mut s = "[".to_string();
        let mut start = true;

        for result in chunks {
            if start {
                start = false;
            } else {
                s.push(',');
            }

            let record = result?;
            let mut json_map = HashMap::new();
            for (i, key) in headers.iter().enumerate() {
                if let Some(value) = record.get(i) {
                    if let Some(t) = types.get(key) {
                        match t {
                            ColumnType::String | ColumnType::DateTime | ColumnType::Enum(_) => {
                                json_map.insert(key, json!(value));
                            }
                            ColumnType::Boolean => {
                                json_map.insert(key, json!(value.parse::<bool>()?));
                            }
                            ColumnType::Int | ColumnType::BigInt => {
                                json_map.insert(key, json!(value.parse::<i64>()?));
                            }
                            ColumnType::Float => {
                                json_map.insert(key, json!(value.parse::<f64>()?));
                            }
                            ColumnType::Decimal => {
                                json_map.insert(key, json!(value.parse::<f64>()?));
                            }
                            ColumnType::Array { .. }
                            | ColumnType::Nested(_)
                            | ColumnType::Json
                            | ColumnType::Bytes => {
                                bail!("CSV importing does not support complex types");
                            }
                        }
                    }
                };
            }
            s.push_str(&serde_json::to_string(&json_map)?);
        }
        s.push(']');

        let res = client
            .post(format!("{}/{}", destination, data_model.data_model.name))
            .body(s)
            .send()
            .await?;

        let status = res.status();
        if status != 200 {
            let body = match res.bytes().await {
                Ok(bytes) => from_utf8_lossy(&bytes).to_string(),
                Err(e) => format!("Getting body failed: {}", e),
            };
            bail!("Import failure with status {}: {}", status, body)
        }
    }
    Ok(())
}
