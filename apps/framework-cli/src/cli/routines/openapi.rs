use crate::framework::core::infrastructure::api_endpoint::APIType;
use crate::framework::core::infrastructure::table::ColumnType;
use crate::framework::core::infrastructure_map::InfrastructureMap;
use crate::project::Project;
use crate::utilities::constants::OPENAPI_FILE;

use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_yaml;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct OpenAPI {
    openapi: String,
    info: Info,
    servers: Vec<Server>,
    paths: HashMap<String, PathItem>,
    components: Components,
}

#[derive(Serialize, Deserialize)]
struct Info {
    title: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
struct Server {
    url: String,
    description: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Parameter {
    name: String,
    #[serde(rename = "in")]
    in_: String,
    required: bool,
    schema: ParameterSchema,
    example: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct ParameterSchema {
    #[serde(rename = "type")]
    schema_type: String,
}

#[derive(Serialize, Deserialize)]
struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    get: Option<Operation>,
}

#[derive(Serialize, Deserialize)]
struct Operation {
    summary: String,
    parameters: Vec<Parameter>,
    #[serde(rename = "requestBody", skip_serializing_if = "Option::is_none")]
    request_body: Option<RequestBody>,
    responses: HashMap<String, Response>,
}

#[derive(Serialize, Deserialize)]
struct RequestBody {
    content: HashMap<String, MediaType>,
}

#[derive(Serialize, Deserialize)]
struct MediaType {
    schema: Schema,
}

#[derive(Serialize, Deserialize, Clone)]
struct Schema {
    #[serde(rename = "type")]
    schema_type: String,
    properties: HashMap<String, Property>,
    required: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Property {
    #[serde(rename = "type")]
    property_type: String,
    example: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct Response {
    description: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    content: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct Components {
    schemas: HashMap<String, Schema>,
}

#[derive(Debug, thiserror::Error)]
pub enum OpenAPIError {
    #[error("Failed to save OpenAPI spec to file: {0}")]
    Save(String),
}

pub async fn openapi(
    project: &Arc<Project>,
    infra_map: &InfrastructureMap,
) -> Result<PathBuf, OpenAPIError> {
    let openapi_spec = generate_openapi_spec(project, infra_map);
    let openapi_file = project.internal_dir().unwrap().join(OPENAPI_FILE);
    save_openapi_to_file(&openapi_spec, &openapi_file.to_string_lossy())
        .map_err(|e| OpenAPIError::Save(e.to_string()))?;

    Ok(openapi_file)
}

fn generate_openapi_spec(project: &Arc<Project>, infra_map: &InfrastructureMap) -> OpenAPI {
    let mut paths = HashMap::new();
    let mut schemas = HashMap::new();

    for api_endpoint in infra_map.api_endpoints.values() {
        match &api_endpoint.api_type {
            APIType::INGRESS {
                data_model: Some(data_model),
                ..
            } => {
                let (properties, required) = data_model.columns.iter().fold(
                    (HashMap::new(), Vec::new()),
                    |(mut props, mut req), column| {
                        let (property_type, example) = map_column_type(&column.data_type);
                        props.insert(
                            column.name.clone(),
                            Property {
                                property_type,
                                example,
                            },
                        );
                        if column.required {
                            req.push(column.name.clone());
                        }
                        (props, req)
                    },
                );

                let schema = Schema {
                    schema_type: "object".to_string(),
                    properties,
                    required,
                };
                schemas.insert(data_model.name.clone(), schema.clone());

                let path_item = PathItem {
                    post: Some(Operation {
                        summary: format!("Ingress endpoint for {}", api_endpoint.name),
                        parameters: vec![],
                        request_body: Some(RequestBody {
                            content: {
                                let mut content = HashMap::new();
                                content
                                    .insert("application/json".to_string(), MediaType { schema });
                                content
                            },
                        }),
                        responses: {
                            let mut responses = HashMap::new();
                            responses.insert(
                                "200".to_string(),
                                Response {
                                    description: "Successful operation".to_string(),
                                    content: HashMap::new(),
                                },
                            );
                            responses
                        },
                    }),
                    get: None,
                };

                paths.insert(
                    format!("/{}", api_endpoint.path.to_string_lossy()),
                    path_item,
                );
            }
            APIType::EGRESS {
                query_params,
                output_schema,
            } => {
                let path_item = PathItem {
                    post: None,
                    get: Some(Operation {
                        summary: format!("Egress endpoint for {}", api_endpoint.name),
                        parameters: query_params
                            .iter()
                            .map(|param| {
                                let (schema_type, example) = map_query_param_type(&param.data_type);
                                Parameter {
                                    name: param.name.clone(),
                                    in_: "query".to_string(),
                                    required: param.required,
                                    schema: ParameterSchema { schema_type },
                                    example,
                                }
                            })
                            .collect(),
                        request_body: None,
                        responses: {
                            let mut responses = HashMap::new();
                            responses.insert(
                                "200".to_string(),
                                Response {
                                    description: "Successful operation".to_string(),
                                    content: HashMap::from([(
                                        "application/json".to_string(),
                                        json!({
                                            "schema": output_schema,
                                        }),
                                    )]),
                                },
                            );
                            responses
                        },
                    }),
                };

                paths.insert(
                    format!("/consumption/{}", api_endpoint.path.to_string_lossy()),
                    path_item,
                );
            }
            _ => {}
        }
    }

    OpenAPI {
        openapi: "3.0.0".to_string(),
        info: Info {
            title: format!("{} API", project.name()),
            version: project.cur_version().to_string(),
        },
        servers: vec![Server {
            url: project.http_server_config.url(),
            description: Some("Server URL".to_string()),
        }],
        paths,
        components: Components { schemas },
    }
}

fn map_column_type(column_type: &ColumnType) -> (String, Option<serde_json::Value>) {
    match column_type {
        ColumnType::Boolean => ("boolean".to_string(), Some(serde_json::Value::Bool(true))),
        ColumnType::Int | ColumnType::BigInt => (
            "integer".to_string(),
            Some(serde_json::Value::Number(1.into())),
        ),
        ColumnType::Float | ColumnType::Decimal => (
            "number".to_string(),
            Some(serde_json::Value::Number(
                serde_json::Number::from_f64(1.0).unwrap(),
            )),
        ),
        ColumnType::DateTime => (
            "string".to_string(),
            Some(serde_json::Value::String(Local::now().to_rfc3339())),
        ),
        ColumnType::Array(_) => (
            "array".to_string(),
            Some(serde_json::Value::Array(vec![serde_json::Value::String(
                "add array items here".to_string(),
            )])),
        ),
        ColumnType::Nested(_) => (
            "object".to_string(),
            Some(serde_json::Value::Object(serde_json::Map::new())),
        ),
        _ => (
            "string".to_string(),
            Some(serde_json::Value::String("stringValue".to_string())),
        ),
    }
}

fn map_query_param_type(data_type: &ColumnType) -> (String, Option<serde_json::Value>) {
    match data_type {
        ColumnType::Boolean => ("boolean".to_string(), Some(serde_json::Value::Bool(true))),
        ColumnType::Int | ColumnType::BigInt => (
            "integer".to_string(),
            Some(serde_json::Value::Number(1.into())),
        ),
        ColumnType::Float | ColumnType::Decimal => (
            "number".to_string(),
            Some(serde_json::Value::Number(
                serde_json::Number::from_f64(1.0).unwrap(),
            )),
        ),
        ColumnType::DateTime => (
            "string".to_string(),
            Some(serde_json::Value::String(Local::now().to_rfc3339())),
        ),
        _ => (
            "string".to_string(),
            Some(serde_json::Value::String("stringValue".to_string())),
        ),
    }
}

fn save_openapi_to_file(openapi_spec: &OpenAPI, file_path: &str) -> std::io::Result<()> {
    let openapi_yaml = serde_yaml::to_string(openapi_spec).unwrap();
    let mut file = File::create(file_path)?;
    file.write_all(openapi_yaml.as_bytes())?;
    Ok(())
}
