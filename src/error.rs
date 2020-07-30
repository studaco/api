use actix_http::ResponseBuilder;
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use bcrypt::BcryptError;
use jwt;
use serde::Serialize;
use sqlx;
use thiserror::Error;

#[derive(Error, Serialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum APIError {
    #[error("Internal server error ({message})")]
    InternalError { message: String },
    #[error("User does not exist")]
    UserDoesNotExist,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Login already present")]
    LoginAlreadyPresent
}

#[derive(Serialize)]
struct ErrorWrapper {
    error: APIError
}

impl ResponseError for APIError {
    fn status_code(&self) -> StatusCode {
        match self {
            APIError::InternalError { message: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            APIError::UserDoesNotExist => StatusCode::NOT_FOUND,
            APIError::InvalidCredentials | APIError::InvalidToken | APIError::LoginAlreadyPresent => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code()).json(ErrorWrapper { error: self.clone() })
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
