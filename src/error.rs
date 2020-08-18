use actix_http::ResponseBuilder;
use actix_web::{
    error::{JsonPayloadError, PathError, QueryPayloadError},
    http::StatusCode,
    HttpRequest, HttpResponse, ResponseError,
};
use bcrypt::BcryptError;
use jwt;
use serde::Serialize;
use sqlx;
use thiserror::Error;

use crate::payload::Payload;

#[derive(Error, Serialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum APIError {
    #[error("Internal server error ({message})")]
    InternalError { message: String },

    #[error("Login already present")]
    LoginAlreadyPresent,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("No token present")]
    NoTokenPresent,
    #[error("Token revoked")]
    TokenRevoked,

    #[error("Bad request")]
    BadRequest {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename="in")]
        scope: Option<RequestScope>,
    },
    #[error("Payload too large")]
    PayloadTooLarge,

    #[error("Lesson does not exist")]
    LessonDosNotExist,

    #[error("No read access")]
    NoReadAccess,
    #[error("No write access")]
    NoWriteAccess,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestScope {
    Body,
    Query,
    Header,
    Path,
}

#[derive(Serialize)]
struct ErrorWrapper {
    error: APIError,
}

impl ResponseError for APIError {
    fn status_code(&self) -> StatusCode {
        match self {
            APIError::InternalError { message: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            APIError::LessonDosNotExist => StatusCode::NOT_FOUND,
            APIError::InvalidCredentials
            | APIError::InvalidToken
            | APIError::TokenExpired
            | APIError::TokenRevoked
            | APIError::NoTokenPresent
            | APIError::LoginAlreadyPresent => StatusCode::UNAUTHORIZED,
            APIError::BadRequest {
                message: _,
                scope: _,
            } => StatusCode::BAD_REQUEST,
            APIError::NoReadAccess | APIError::NoWriteAccess => StatusCode::FORBIDDEN,
            APIError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        }
    }

    fn error_response(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code()).json(ErrorWrapper {
            error: self.clone(),
        })
    }
}

impl From<sqlx::Error> for APIError {
    fn from(error: sqlx::Error) -> Self {
        log::debug!("{:?}", error);
        APIError::InternalError {
            message: format!("{}", error),
        }
    }
}

impl From<BcryptError> for APIError {
    fn from(error: BcryptError) -> Self {
        APIError::InternalError {
            message: format!("{}", error),
        }
    }
}

impl From<jwt::Error> for APIError {
    fn from(_: jwt::Error) -> Self {
        APIError::InvalidToken
    }
}

trait IntoAPI {
    fn internal_error(self) -> APIError;
}

impl<T: std::error::Error> IntoAPI for T {
    fn internal_error(self) -> APIError {
        APIError::InternalError {
            message: format!("{}", self),
        }
    }
}
trait IntoAPIResult<T> {
    fn internal_error(self) -> std::result::Result<T, APIError>;
}

impl<T, E: std::error::Error> IntoAPIResult<T> for std::result::Result<T, E> {
    fn internal_error(self) -> std::result::Result<T, APIError> {
        self.map_err(|err| err.internal_error())
    }
}

pub type Result<T> = std::result::Result<Payload<T>, APIError>;

pub fn json_error_handler(error: JsonPayloadError, _: &HttpRequest) -> actix_web::Error {
    match error {
        JsonPayloadError::Overflow => APIError::PayloadTooLarge,
        JsonPayloadError::Deserialize(error) => APIError::BadRequest {
            message: format!("{}", error),
            scope: Some(RequestScope::Body),
        },
        _ => APIError::BadRequest {
            message: format!("{}", error),
            scope: Some(RequestScope::Body),
        },
    }
    .into()
}

pub fn path_error_handler(
    PathError::Deserialize(error): PathError,
    _: &HttpRequest,
) -> actix_web::Error {
    APIError::BadRequest {
        message: format!("{}", error),
        scope: Some(RequestScope::Path),
    }
    .into()
}

pub fn query_error_handler(
    QueryPayloadError::Deserialize(error): QueryPayloadError,
    _: &HttpRequest
) -> actix_web::Error {
    APIError::BadRequest {
        message: format!("{}", error),
        scope: Some(RequestScope::Query)
    }.into()
}