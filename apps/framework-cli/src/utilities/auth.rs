use hyper::header::HeaderValue;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::project::JwtConfig;

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    iss: String,
    aud: String,
    exp: i64,
}

fn create_validation(issuer: &str, audience: &str) -> Validation {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[issuer]);
    validation.set_audience(&[audience]);
    validation.set_required_spec_claims(&["iss", "aud", "exp"]);
    validation
}

fn decode_token<T: for<'de> Deserialize<'de>>(
    token: &str,
    public_key: &str,
    validation: &Validation,
) -> Result<T, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_rsa_pem(public_key.as_ref())?;
    decode::<T>(token, &decoding_key, validation).map(|data| data.claims)
}

pub fn validate_jwt(token: Option<&str>, public_key: &str, issuer: &str, audience: &str) -> bool {
    token.is_some_and(|t| {
        let validation = create_validation(issuer, audience);
        decode_token::<Claims>(t, public_key, &validation).is_ok()
    })
}

pub fn get_claims(
    auth_header: Option<&HeaderValue>,
    jwt_config: &Option<JwtConfig>,
) -> Option<Value> {
    let bearer_token = auth_header
        .and_then(|header_value| header_value.to_str().ok())
        .and_then(|header_str| header_str.strip_prefix("Bearer "));

    if let (Some(token), Some(config)) = (bearer_token, jwt_config) {
        let validation = create_validation(&config.issuer, &config.audience);
        if let Ok(claims) = decode_token::<Value>(token, &config.secret, &validation) {
            return Some(claims);
        }
    }

    None
}
