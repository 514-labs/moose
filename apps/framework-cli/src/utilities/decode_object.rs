use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;

pub fn decode_base64_to_json(encoded: &str) -> Result<Value, String> {
    general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| format!("Invalid base64 encoding: {}", e))
        .and_then(|json_str| {
            serde_json::from_slice::<Value>(&json_str).map_err(|e| format!("Invalid JSON: {}", e))
        })
}
