use actix_http::http::HeaderMap;
use hmac::{Hmac, NewMac};
use jwt::{Error, SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::env;
use std::vec::Vec;
use uuid::Uuid;

use crate::error::APIError;

#[derive(Serialize, Deserialize)]
struct Claim {
    id: Uuid,
}

pub fn generate_token(id: Uuid) -> Result<String, Error> {
    let token_secret: String = env::var("TOKEN_SECRET").expect("TOKEN_SECRET is not set");
    let token_key: Hmac<Sha256> = Hmac::new_varkey(&token_secret.into_bytes()[..]).unwrap();
    let claim = Claim { id };
    claim.sign_with_key(&token_key)
}

pub fn authorize_headers(headers: &HeaderMap) -> Result<Uuid, APIError> {
    let header_value = headers
        .get("Authorization")
        .ok_or(APIError::NoTokenPresent)?;

    let header = header_value.to_str().map_err(|err| APIError::BadRequest {
        message: Some(format!("{}", err)),
    })?;

    let values: Vec<&str> = header.split_ascii_whitespace().collect();
    if values.len() != 2 {
        return Err(APIError::InvalidToken);
    }
    if values[0].to_lowercase() != "bearer" {
        return Err(APIError::InvalidToken);
    }

    let token = values[1];

    let token_secret: String = env::var("TOKEN_SECRET").expect("TOKEN_SECRET is not set");
    let token_key: Hmac<Sha256> = Hmac::new_varkey(&token_secret.into_bytes()[..]).unwrap();
    let Claim { id } = token
        .verify_with_key(&token_key)
        .map_err(|_| APIError::InvalidToken)?;

    Ok(id)
}
