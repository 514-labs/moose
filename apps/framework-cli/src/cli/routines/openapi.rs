use crate::framework::core::infrastructure::api_endpoint::APIType;
use crate::framework::core::infrastructure::table::ColumnType;
use crate::framework::core::infrastructure_map::InfrastructureMap;
use crate::infrastructure::olap::clickhouse_alt_client::{get_pool, retrieve_infrastructure_map};
use crate::project::Project;
use crate::utilities::constants::OPENAPI_FILE;

use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
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
struct PathItem {
    #[serde(rename = "post")]
    post: Option<Operation>,
}

#[derive(Serialize, Deserialize)]
struct Operation {
    summary: String,
    #[serde(rename = "requestBody")]
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
}

#[derive(Serialize, Deserialize)]
struct Components {
    schemas: HashMap<String, Schema>,
}

#[derive(Debug, thiserror::Error)]
pub enum OpenAPIError {
    #[error("Error connecting to storage")]
    Connection,
    #[error("Error retrieving current state")]
    Retrieval,
    #[error("No state found")]
    NoState,
    #[error("Failed to save OpenAPI spec to file: {0}")]
    Save(String),
}

pub async fn openapi(project: &Arc<Project>) -> Result<(), OpenAPIError> {
    let mut client = get_pool(&project.clickhouse_config)
        .get_handle()
        .await
        .map_err(|_| OpenAPIError::Connection)?;

    let infra = retrieve_infrastructure_map(&mut client, &project.clickhouse_config)
        .await
        .map_err(|_| OpenAPIError::Retrieval)?
        .ok_or(OpenAPIError::NoState)?;

    let openapi_spec = generate_openapi_spec(project, &infra);
    let openapi_file = project.internal_dir().unwrap().join(OPENAPI_FILE);
    save_openapi_to_file(&openapi_spec, &openapi_file.to_string_lossy())
        .map_err(|e| OpenAPIError::Save(e.to_string()))?;

    Ok(())
}

fn generate_openapi_spec(project: &Arc<Project>, infra_map: &InfrastructureMap) -> OpenAPI {
    let mut paths = HashMap::new();
    let mut schemas = HashMap::new();

    for api_endpoint in infra_map.api_endpoints.values() {
        if let APIType::INGRESS {
            data_model: Some(data_model),
            ..
        } = &api_endpoint.api_type
        {
            let (properties, required) = data_model.columns.iter().fold(
                (HashMap::new(), Vec::new()),
                |(mut props, mut req), column| {
                    let (property_type, example) = match column.data_type {
                        ColumnType::Boolean => {
                            ("boolean".to_string(), Some(serde_json::Value::Bool(true)))
                        }
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
                                "example".to_string(),
                            )])),
                        ),
                        _ => (
                            "string".to_string(),
                            Some(serde_json::Value::String("stringValue".to_string())),
                        ),
                    };

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
                    request_body: Some(RequestBody {
                        content: {
                            let mut content = HashMap::new();
                            content.insert("application/json".to_string(), MediaType { schema });
                            content
                        },
                    }),
                    responses: {
                        let mut responses = HashMap::new();
                        responses.insert(
                            "200".to_string(),
                            Response {
                                description: "Successful operation".to_string(),
                            },
                        );
                        responses
                    },
                }),
            };

            paths.insert(
                format!("/{}", api_endpoint.path.to_string_lossy()),
                path_item,
            );
        }
    }

    OpenAPI {
        openapi: "3.0.0".to_string(),
        info: Info {
            title: "Generated API".to_string(),
            version: "1.0.0".to_string(),
        },
        servers: vec![Server {
            url: project.http_server_config.url(),
            description: Some("Ingress endpoint".to_string()),
        }],
        paths,
        components: Components { schemas },
    }
}

fn save_openapi_to_file(openapi_spec: &OpenAPI, file_path: &str) -> std::io::Result<()> {
    let openapi_yaml = serde_yaml::to_string(openapi_spec).unwrap();
    let mut file = File::create(file_path)?;
    file.write_all(openapi_yaml.as_bytes())?;
    Ok(())
}
