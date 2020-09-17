use sqlx;
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::error::APIError;

#[derive(Debug, sqlx::Type)]
#[sqlx(rename = "permissiontype", rename_all = "lowercase")]
pub enum PgPermissionType {
    R,
    RW,
}

impl From<PermissionType> for PgPermissionType {
    fn from(pt: PermissionType) -> Self {
        match pt {
            PermissionType::Read => PgPermissionType::R,
            PermissionType::ReadWrite => PgPermissionType::RW,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Eq, PartialEq)]
pub enum PermissionType {
    #[serde(rename = "r")]
    Read,
    #[serde(rename = "rw")]
    ReadWrite,
}

impl From<PgPermissionType> for PermissionType {
    fn from(pt: PgPermissionType) -> Self {
        match pt {
            PgPermissionType::R => PermissionType::Read,
            PgPermissionType::RW => PermissionType::ReadWrite,
        }
    }
}

#[derive(Error, Debug)]
pub enum PermissionError {
    #[error("Entity not present")]
    EntityNotPresent,
    #[error("Permission not present")]
    PermissionNotPresent,
    #[error("sqlx error occurred while fetching permission ({0})")]
    Sqlx(#[from] sqlx::Error),
}

impl From<PermissionError> for APIError {
    fn from(error: PermissionError) -> APIError {
        match error {
            PermissionError::EntityNotPresent => APIError::LessonDosNotExist, // TODO: Store entity type in the EntityNotPresent
            PermissionError::PermissionNotPresent => APIError::NoReadAccess,
            PermissionError::Sqlx(error) => error.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, PermissionError>;
