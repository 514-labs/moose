use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;

pub fn decode_base64_to_json(encoded: &str) -> Value {
    general_purpose::STANDARD
        .decode(encoded)
        .ok()
        .and_then(|json_str| serde_json::from_slice::<Value>(&json_str).ok())
        .unwrap_or_else(|| serde_json::json!({}))
}
