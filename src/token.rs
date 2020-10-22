use actix_http::http::HeaderMap;
use chrono::Duration;
use hmac::{Hmac, NewMac};
use jwt::{SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::Sha256;
use std::env;
use std::vec::Vec;
use thiserror::Error;
use futures::future::{ready, Ready};

use crate::error::{APIError, RequestScope};
use crate::model::account::AccountID;
use crate::uuid_wrapper;

pub type SecondsSinceEpoch = i64;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct RegisteredClaims {
    #[serde(rename = "iss", skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,

    #[serde(rename = "sub", skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,

    #[serde(rename = "aud", skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,

    #[serde(rename = "exp")]
    pub expiration: SecondsSinceEpoch,

    #[serde(rename = "nbf", skip_serializing_if = "Option::is_none")]
    pub not_before: Option<SecondsSinceEpoch>,

    #[serde(rename = "iat", skip_serializing_if = "Option::is_none")]
    pub issued_at: Option<SecondsSinceEpoch>,

    #[serde(rename = "jti", skip_serializing_if = "Option::is_none")]
    pub json_web_token_id: Option<String>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApplicationClaim<T> {
    #[serde(flatten)]
    pub inner: T,

    #[serde(flatten)]
    pub registered: RegisteredClaims,
}

uuid_wrapper!(RefreshTokenID);

impl<T> ApplicationClaim<T> {
    pub fn new(inner: T, valid_for: Duration) -> Result<ApplicationClaim<T>, InvalidDuration> {
        let now = chrono::Utc::now();
        let expiration = now
            .checked_add_signed(valid_for)
            .ok_or(InvalidDuration {})?
            .timestamp();
        Ok(ApplicationClaim {
            inner,
            registered: RegisteredClaims {
                expiration,
                issued_at: Some(now.timestamp()),
                ..RegisteredClaims::default()
            },
        })
    }
}

impl<T> actix_web::FromRequest for ApplicationClaim<T> where T: Clone + 'static {
    type Error = APIError;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_http::Payload) -> Self::Future {
        ready(
            req.extensions()
                .get::<Self>()
                .map(Self::clone)
                .ok_or(APIError::InternalError {
                    message: "Error encountered while extracting parameters".to_string(),
                }),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub struct AccessTokenInfo {
    pub account_id: AccountID,
    /// Refresh token id used to issue this access token
    pub token_origin: RefreshTokenID,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub struct RefreshTokenInfo {
    pub account_id: AccountID,
    pub token_id: RefreshTokenID,
}

pub trait ApplicationToken: for <'de> Deserialize<'de> + Serialize + From<String> {
    // type Claim: Serialize + Deserialize<'static>;
    type Claim: Serialize + DeserializeOwned;
    fn valid_for() -> Duration;
    fn str_ref(&self) -> &str;

    fn generate_token(info: Self::Claim) -> Result<Self, APIError> {
        let token_secret: String = env::var("TOKEN_SECRET").expect("TOKEN_SECRET is not set");
        let token_key: Hmac<Sha256> = Hmac::new_varkey(&token_secret.into_bytes()[..]).unwrap();
        let claim = ApplicationClaim::new(info, Self::valid_for())?;
        let token = claim.sign_with_key(&token_key)?;
        Ok(token.into())
    }

    fn authenticate_claim(&self) -> Result<ApplicationClaim<Self::Claim>, APIError> {
        let token_secret: String = env::var("TOKEN_SECRET").expect("TOKEN_SECRET is not set");
        let token_key: Hmac<Sha256> = Hmac::new_varkey(&token_secret.into_bytes()[..]).unwrap();
        let claim: ApplicationClaim<Self::Claim> = self.str_ref()
            .verify_with_key(&token_key)
            .map_err(|_| APIError::InvalidToken)?;
        let now = chrono::Utc::now().timestamp();
        if claim.registered.expiration < now {
            return Err(APIError::TokenExpired)
        }
        Ok(claim)
    }

}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct AccessToken(String);

impl std::fmt::Display for AccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for AccessToken {
    fn from(str: String) -> Self {
        Self(str)
    }
}

impl ApplicationToken for AccessToken {
    type Claim = AccessTokenInfo;
    fn valid_for() -> Duration { Duration::minutes(5) }
    fn str_ref(&self) -> &str { &self.0[..] }
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct RefreshToken(String);

impl From<String> for RefreshToken {
    fn from(str: String) -> Self {
        Self(str)
    }
}

impl ApplicationToken for RefreshToken {
    type Claim = RefreshTokenInfo;
    fn valid_for() -> Duration { Duration::days(14) }
    fn str_ref(&self) -> &str { &self.0[..] }
}

#[derive(Error, Debug)]
#[error("Invalid token validity duration. Datetime overflow")]
pub struct InvalidDuration {}

impl From<InvalidDuration> for APIError {
    fn from(error: InvalidDuration) -> APIError {
        APIError::InternalError {
            message: format!("{}", error),
        }
    }
}

pub fn generate_token_pair(account_id: AccountID) -> Result<(AccessToken, RefreshToken), APIError> {
    let token_id = RefreshTokenID(uuid::Uuid::new_v4());
    let refresh_token = RefreshToken::generate_token(RefreshTokenInfo { account_id, token_id })?;
    let access_token = AccessToken::generate_token(AccessTokenInfo { account_id, token_origin: token_id })?;
    return Ok((access_token, refresh_token));
}

pub fn authenticate_claim(AccessToken(token): AccessToken) -> Result<ApplicationClaim<AccessTokenInfo>, APIError> {
    let token_secret: String = env::var("TOKEN_SECRET").expect("TOKEN_SECRET is not set");
    let token_key: Hmac<Sha256> = Hmac::new_varkey(&token_secret.into_bytes()[..]).unwrap();
    let claim: ApplicationClaim<AccessTokenInfo> = token
        .verify_with_key(&token_key)
        .map_err(|_| APIError::InvalidToken)?;
    let now = chrono::Utc::now().timestamp();
    if claim.registered.expiration < now {
        return Err(APIError::TokenExpired)
    }
    Ok(claim)
}

pub fn authenticate_claim_from_headers(headers: &HeaderMap) -> Result<ApplicationClaim<AccessTokenInfo>, APIError> {
    extract_token(headers).and_then(authenticate_claim)
}

pub fn extract_token(headers: &HeaderMap) -> Result<AccessToken, APIError> {
    let header_value = headers
        .get("Authorization")
        .ok_or(APIError::NoTokenPresent)?;

    let header = header_value.to_str().map_err(|err| APIError::BadRequest {
        message: format!("{}", err),
        scope: Some(RequestScope::Header),
    })?;

    let values: Vec<&str> = header.split_ascii_whitespace().collect();
    if values.len() != 2 {
        return Err(APIError::InvalidToken);
    }
    if values[0].to_lowercase() != "bearer" {
        return Err(APIError::InvalidToken);
    }

    // Not a fan of an allocation here
    Ok(AccessToken(values[1].to_string()))
}
