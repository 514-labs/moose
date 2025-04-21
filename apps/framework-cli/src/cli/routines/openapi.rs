use crate::framework::consumption::model::ConsumptionQueryParam;
use crate::framework::core::infrastructure::api_endpoint::{APIType, ApiEndpoint};
use crate::framework::core::infrastructure::table::{Column, ColumnType};
use crate::framework::core::infrastructure_map::InfrastructureMap;
use crate::framework::data_model::model::DataModel;
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
    required: bool,
    content: HashMap<String, MediaType>,
}

#[derive(Serialize, Deserialize)]
struct MediaType {
    schema: MediaTypeSchema,
}

#[derive(Serialize, Deserialize, Clone)]
struct MediaTypeSchema {
    #[serde(rename = "$ref")]
    ref_: String,
}

#[derive(Serialize, Deserialize)]
struct Schema {
    #[serde(rename = "type")]
    schema_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<HashMap<String, Property>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Property {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    property_type: Option<String>,
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    ref_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    example: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<PropertyItem>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum OpenApiType {
    Simple(String),
    Complex(Value),
}

fn nullable(simple_type: String) -> OpenApiType {
    OpenApiType::Complex(json!({
        "oneOf": [
            {"type": "null"},
            {"type": simple_type},
        ]
    }))
}

#[derive(Serialize, Deserialize)]
struct PropertyItem {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    type_: Option<OpenApiType>,
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    ref_: Option<String>,
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
        if let APIType::INGRESS {
            data_model: Some(data_model),
            ..
        } = &api_endpoint.api_type
        {
            build_data_model_schema(data_model, &mut schemas);
        }
    }

    for api_endpoint in infra_map.api_endpoints.values() {
        match &api_endpoint.api_type {
            APIType::INGRESS {
                data_model: Some(data_model),
                ..
            } => {
                let path_item = create_ingress_path_item(api_endpoint, data_model);
                paths.insert(
                    format!("/{}", api_endpoint.path.to_string_lossy()),
                    path_item,
                );
            }

            APIType::EGRESS {
                query_params,
                output_schema,
            } => {
                let (path_item, component_schemas) =
                    create_egress_path_item(api_endpoint, output_schema.clone(), query_params);
                paths.insert(
                    format!("/consumption/{}", api_endpoint.path.to_string_lossy()),
                    path_item,
                );
                // Merge egress schemas into the main schemas map
                schemas.extend(component_schemas);
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

fn create_ingress_path_item(api_endpoint: &ApiEndpoint, data_model: &DataModel) -> PathItem {
    PathItem {
        post: Some(Operation {
            summary: format!("Ingress endpoint for {}", api_endpoint.name),
            parameters: vec![],
            request_body: Some(RequestBody {
                required: true,
                content: {
                    let mut content = HashMap::new();
                    content.insert(
                        "application/json".to_string(),
                        MediaType {
                            schema: MediaTypeSchema {
                                ref_: format!("#/components/schemas/{}", data_model.name),
                            },
                        },
                    );
                    content
                },
            }),
            responses: create_default_responses(),
        }),
        get: None,
    }
}

fn create_egress_path_item(
    api_endpoint: &ApiEndpoint,
    output_schema: Value,
    query_params: &[ConsumptionQueryParam],
) -> (PathItem, HashMap<String, Schema>) {
    let default_schema = json!({"type": "object"});
    let (response_schema, component_schemas) = if output_schema != Value::Null {
        extract_component_schemas(output_schema)
    } else {
        (default_schema, HashMap::new())
    };

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
            responses: HashMap::from([(
                "200".to_string(),
                Response {
                    description: "Successful operation".to_string(),
                    content: HashMap::from([(
                        "application/json".to_string(),
                        json!({ "schema": response_schema }),
                    )]),
                },
            )]),
        }),
    };

    (path_item, component_schemas)
}

fn extract_component_schemas(schema: Value) -> (Value, HashMap<String, Schema>) {
    let mut component_schemas = HashMap::new();

    // Handle typia-style schema
    if let Some(components) = schema.get("components").and_then(|c| c.get("schemas")) {
        if let Some(obj) = components.as_object() {
            // Copy all schemas directly to top level component schemas
            for (name, schema_def) in obj {
                component_schemas.insert(
                    name.clone(),
                    serde_json::from_value(schema_def.clone())
                        .unwrap_or_else(|_| panic!("Failed to deserialize schema for {}", name)),
                );
            }
        }

        // Reference the schema in the response
        if let Some(schemas) = schema.get("schemas").and_then(|s| s.as_array()) {
            if let Some(first_schema) = schemas.first() {
                return (first_schema.clone(), component_schemas);
            }
        }
    }

    // Handle pydantic-style schema
    if let Some(defs) = schema.get("$defs") {
        if let Some(obj) = defs.as_object() {
            // Copy all schemas directly to top level component schemas
            for (name, schema_def) in obj {
                component_schemas.insert(
                    name.clone(),
                    serde_json::from_value(schema_def.clone())
                        .unwrap_or_else(|_| panic!("Failed to deserialize schema for {}", name)),
                );
            }
        }

        // Reference the schema in the response
        let mut response_schema = schema.clone();
        response_schema
            .as_object_mut()
            .and_then(|obj| obj.remove("$defs"));
        return (response_schema, component_schemas);
    }

    (schema, component_schemas)
}

fn create_default_responses() -> HashMap<String, Response> {
    let mut responses = HashMap::new();
    responses.insert(
        "200".to_string(),
        Response {
            description: "Successful operation".to_string(),
            content: HashMap::new(),
        },
    );
    responses
}

fn build_data_model_schema(data_model: &DataModel, schemas: &mut HashMap<String, Schema>) {
    build_schema(&data_model.columns, data_model.name.clone(), schemas);
}

fn build_schema(columns: &Vec<Column>, parent_name: String, schemas: &mut HashMap<String, Schema>) {
    let mut properties = HashMap::new();
    let mut required = Vec::new();

    for column in columns {
        let property = match &column.data_type {
            ColumnType::Nested(fields) => {
                let component_name = format!("{}_{}", parent_name, column.name);
                build_schema(&fields.columns, component_name.clone(), schemas);
                Property {
                    property_type: None,
                    ref_: Some(format!("#/components/schemas/{}", component_name)),
                    example: None,
                    items: None,
                }
            }
            ColumnType::Array {
                element_type: column_type,
                element_nullable,
            } => {
                let item_type = if let ColumnType::Nested(fields) = &**column_type {
                    let component_name = format!("{}_{}", parent_name, column.name);
                    build_schema(&fields.columns, component_name.clone(), schemas);
                    PropertyItem {
                        type_: None,
                        ref_: Some(format!("#/components/schemas/{}", component_name)),
                    }
                } else {
                    let (property_type, _) = map_column_type(column_type);
                    let t = if *element_nullable {
                        nullable(property_type)
                    } else {
                        OpenApiType::Simple(property_type)
                    };
                    PropertyItem {
                        type_: Some(t),
                        ref_: None,
                    }
                };
                Property {
                    property_type: Some("array".to_string()),
                    ref_: None,
                    example: None,
                    items: Some(item_type),
                }
            }
            _ => {
                let (property_type, example) = map_column_type(&column.data_type);
                Property {
                    property_type: Some(property_type),
                    ref_: None,
                    example,
                    items: None,
                }
            }
        };

        properties.insert(column.name.clone(), property);

        if column.required {
            required.push(column.name.clone());
        }
    }

    schemas.insert(
        parent_name.clone(),
        Schema {
            schema_type: "object".to_string(),
            properties: Some(properties),
            required: Some(required),
            title: None,
        },
    );
}

fn map_column_type(column_type: &ColumnType) -> (String, Option<serde_json::Value>) {
    match column_type {
        ColumnType::Boolean => ("boolean".to_string(), Some(serde_json::Value::Bool(true))),
        ColumnType::Int(_) | ColumnType::BigInt => (
            "integer".to_string(),
            Some(serde_json::Value::Number(1.into())),
        ),
        ColumnType::Float(_) | ColumnType::Decimal { .. } => (
            "number".to_string(),
            Some(serde_json::Value::Number(
                serde_json::Number::from_f64(1.0).unwrap(),
            )),
        ),
        ColumnType::DateTime { .. } => (
            "string".to_string(),
            Some(serde_json::Value::String(Local::now().to_rfc3339())),
        ),
        ColumnType::Array { .. } => (
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
        ColumnType::Int(_) | ColumnType::BigInt => (
            "integer".to_string(),
            Some(serde_json::Value::Number(1.into())),
        ),
        ColumnType::Float(_) | ColumnType::Decimal { .. } => (
            "number".to_string(),
            Some(serde_json::Value::Number(
                serde_json::Number::from_f64(1.0).unwrap(),
            )),
        ),
        ColumnType::DateTime { .. } => (
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
