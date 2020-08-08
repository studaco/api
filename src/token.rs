use actix_http::{Payload, http::HeaderMap};
use actix_web::{FromRequest, HttpRequest};
use hmac::{Hmac, NewMac};
use jwt::{Error, SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::env;
use std::vec::Vec;
use uuid::Uuid;
use futures::future::{ready, Ready};

use crate::error::APIError;

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Claim {
    pub id: Uuid,
}

impl FromRequest for Claim {
    type Error = APIError;
    type Future = Ready<Result<Claim, APIError>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(
            req.extensions()
                .get::<Claim>()
                .map(|claim| claim.clone())
                .ok_or(APIError::InternalError {
                    message: "Error encountered while processing authentication request"
                        .to_string(),
                }),
        )
    }
}


pub fn generate_token(id: Uuid) -> Result<String, Error> {
    let token_secret: String = env::var("TOKEN_SECRET").expect("TOKEN_SECRET is not set");
    let token_key: Hmac<Sha256> = Hmac::new_varkey(&token_secret.into_bytes()[..]).unwrap();
    let claim = Claim { id };
    claim.sign_with_key(&token_key)
}

pub fn authenticate_claim(token: &str) -> Result<Claim, APIError> {
    let token_secret: String = env::var("TOKEN_SECRET").expect("TOKEN_SECRET is not set");
    let token_key: Hmac<Sha256> = Hmac::new_varkey(&token_secret.into_bytes()[..]).unwrap();
    Ok(token
        .verify_with_key(&token_key)
        .map_err(|_| APIError::InvalidToken)?)
}

pub fn authenticate_claim_from_headers(headers: &HeaderMap) -> Result<Claim, APIError> {
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

    authenticate_claim(values[1])
}