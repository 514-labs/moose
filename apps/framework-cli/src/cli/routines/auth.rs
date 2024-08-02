use openssl::rand::rand_bytes;
use pbkdf2::pbkdf2_hmac;

use sha2::Sha256;

use crate::cli::display::{Message, MessageType};

pub fn generate_hash_token() {
    // split token on the dot delimiter which separates the token and the salt
    let n = 1000;
    let mut token = [0u8; 16];
    let mut salt = [0u8; 16];
    rand_bytes(&mut token).unwrap();
    rand_bytes(&mut salt).unwrap();
    // Convert to hexadecimal strings
    let token_hex = hex::encode(token);
    let salt_hex = hex::encode(salt);

    // Concatenate token and salt with a '.' delimiter
    let concatenated = format!("{}.{}", token_hex, salt_hex);

    // set timer to c

    let mut key1 = [0u8; 20];
    pbkdf2_hmac::<Sha256>(&token, &salt, n, &mut key1);

    {
        show_message!(
            MessageType::Info,
            Message {
                action: "ENV API Keys".to_string(),
                details: format!(
                    "{} \n MOOSE_INGEST_API_KEY / MOOSE_CONSUMPTION_API_KEY",
                    hex::encode(key1)
                ),
            }
        );
    }

    {
        show_message!(
            MessageType::Info,
            Message {
                action: "Bearer Token".to_string(),
                details: format!("{} \n Authentication bearer token", concatenated),
            }
        );
    }
}

pub fn validate_auth_token(token: &str, private_pass: &str) -> bool {
    let token_parts: Vec<&str> = token.split('.').collect();
    if token_parts.len() != 2 {
        return false;
    }

    let token_hex = match hex::decode(token_parts[0]) {
        Ok(token) => token,
        Err(_) => return false,
    };

    let salt_hex = match hex::decode(token_parts[1]) {
        Ok(salt) => salt,
        Err(_) => return false,
    };

    let mut key1 = [0u8; 20];
    pbkdf2_hmac::<Sha256>(&token_hex, &salt_hex, 1000, &mut key1);

    let key1_hex = hex::encode(key1);

    // compare byte to byte to avoid timing attacks
    let token_hash = hex::decode(private_pass).unwrap();
    let key1_hash = hex::decode(key1_hex).unwrap();

    for i in 0..token_hash.len() {
        if token_hash[i] != key1_hash[i] {
            return false;
        }
    }
    return true;
}
