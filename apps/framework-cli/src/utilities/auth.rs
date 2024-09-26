use jsonwebtoken::{decode, Algorithm, DecodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    iss: String,
    aud: String,
}

pub fn validate_jwt(token: Option<&str>, public_key: &str, issuer: &str, audience: &str) -> bool {
    token.map_or(false, |t| {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[issuer]);
        validation.set_audience(&[audience]);
        validation.set_required_spec_claims(&["iss", "aud"]);
        validation.validate_exp = false;

        match DecodingKey::from_rsa_pem(public_key.as_ref()) {
            Ok(decoding_key) => decode::<Claims>(t, &decoding_key, &validation).is_ok(),
            Err(_) => false,
        }
    })
}
