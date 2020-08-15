use actix_http::{http::HeaderMap};
use hmac::{Hmac, NewMac};
use jwt::{Error, SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::env;
use std::vec::Vec;
use uuid::Uuid;

use crate::error::{APIError, RequestScope};
use crate::model::account::AccountID;

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Claim {
    pub id: Uuid,
}

impl From<AccountID> for Claim {
    fn from(AccountID(id): AccountID) -> Claim {
        Claim { id }
    }
}

pub fn generate_token(id: AccountID) -> Result<String, Error> {
    let token_secret: String = env::var("TOKEN_SECRET").expect("TOKEN_SECRET is not set");
    let token_key: Hmac<Sha256> = Hmac::new_varkey(&token_secret.into_bytes()[..]).unwrap();
    let claim: Claim = id.into();
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
        message: format!("{}", err),
        scope: Some(RequestScope::Header)
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
