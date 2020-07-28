use actix_http::ResponseBuilder;
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use bcrypt::BcryptError;
use jwt;
use serde::Serialize;
use sqlx;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum APIError {
    InternalError { message: Option<String> },
    UserDoesNotExist,
    InvalidCredentials,
    InvalidToken,
}

impl Display for APIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            APIError::InternalError { message } => match message {
                Some(str) => write!(f, "Internal server error ({})", str),
                None => f.write_str("Internal server error"),
            },
            APIError::UserDoesNotExist => write!(f, "User deos not exist"),
            APIError::InvalidCredentials => write!(f, "Invalid creatials"),
            APIError::InvalidToken => f.write_str("Invalid token"),
        }
    }
}

impl std::error::Error for APIError {}

impl ResponseError for APIError {
    fn status_code(&self) -> StatusCode {
        match self {
            APIError::InternalError { message: _ } => StatusCode::INTERNAL_SERVER_ERROR,
            APIError::UserDoesNotExist => StatusCode::NOT_FOUND,
            APIError::InvalidCredentials | APIError::InvalidToken => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code()).json(self)
    }
}

impl From<sqlx::Error> for APIError {
    fn from(error: sqlx::Error) -> Self {
        APIError::InternalError {
            message: Some(format!("{}", error)),
        }
    }
}

impl From<BcryptError> for APIError {
    fn from(error: BcryptError) -> Self {
        APIError::InternalError {
            message: Some(format!("{}", error)),
        }
    }
}

impl From<jwt::Error> for APIError {
    fn from(_: jwt::Error) -> Self {
        APIError::InvalidToken
    }
}
