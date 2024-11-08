#[derive(Debug, thiserror::Error)]
#[error("failed interact with clickhouse")]
#[non_exhaustive]
pub enum ClickhouseError {
    #[error("Clickhouse - Unsupported data type: {type_name}")]
    UnsupportedDataType {
        type_name: String,
    },
    #[error("Clickhouse - Invalid parameters: {message}")]
    InvalidParameters {
        message: String,
    },
    QueryRender(#[from] handlebars::RenderError),
}
