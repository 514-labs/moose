#[derive(Debug, thiserror::Error)]
#[error("failed interact with clickhouse")]
#[non_exhaustive]
pub enum ClickhouseError {
    #[error("Clickhouse - Unsupported data type: {type_name}")]
    UnsupportedDataType {
        type_name: String,
    },
    QueryRender(#[from] handlebars::RenderError),
}
