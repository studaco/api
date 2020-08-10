use actix_http::ResponseBuilder;
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
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
    BadRequest { message: Option<String> },

    #[error("Lesson does not exist")]
    LessonDosNotExist,

    #[error("No read access")]
    NoReadAccess,
    #[error("No write access")]
    NoWriteAccess,
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
            APIError::BadRequest { message: _ } => StatusCode::BAD_REQUEST,
            APIError::NoReadAccess | APIError::NoWriteAccess => StatusCode::FORBIDDEN,
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