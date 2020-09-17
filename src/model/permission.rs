use actix_http::Payload;
use actix_web::{FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{PgPool, PgQueryAs, PgRow},
    Row,
};
use thiserror::Error;

use super::account::AccountID;
use super::lesson::LessonID;
use super::Transaction;
use crate::error::APIError;

#[derive(Debug, sqlx::Type)]
#[sqlx(rename = "permissiontype", rename_all = "lowercase")]
enum PgPermissionType {
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

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct LessonPermission {
    pub permission_type: PermissionType,
    pub lesson_id: LessonID,
    pub account_id: AccountID,
}

impl FromRequest for LessonPermission {
    type Error = APIError;
    type Future = Ready<std::result::Result<LessonPermission, APIError>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(
            req.extensions()
                .get::<LessonPermission>()
                .map(|id| id.clone())
                .ok_or(APIError::InternalError {
                    message: "Error encountered while processing permission checks"
                        .to_string(),
                }),
        )
    }
}


#[derive(Debug, Error)]
#[error("Invalid Permission Type. Should be either \"r\" or \"rw\"")]
struct InvalidPermissionType {}

// How to make this one generic over database types?
impl<'c> sqlx::FromRow<'c, PgRow<'c>> for LessonPermission {
    fn from_row(row: &PgRow<'c>) -> sqlx::Result<Self> {
        let permission_type: PgPermissionType = row.try_get("type")?;
        let lesson_id: LessonID = row.try_get("lesson_id")?;
        let account_id: AccountID = row.try_get("account_id")?;

        Ok(LessonPermission {
            permission_type: permission_type.into(),
            lesson_id,
            account_id,
        })
    }
}

#[derive(Error, Debug)]
pub enum PermissionError {
    #[error("Entity not present")]
    LessonNotPresent,
    #[error("Permission not present")]
    PermissionNotPresent,
    #[error("sqlx error occurred while fetching permission ({0})")]
    Sqlx(#[from] sqlx::Error),
}

impl From<PermissionError> for APIError {
    fn from(error: PermissionError) -> APIError {
        match error {
            PermissionError::LessonNotPresent => APIError::LessonDosNotExist,
            PermissionError::PermissionNotPresent => APIError::NoReadAccess,
            PermissionError::Sqlx(error) => error.into(),
        }
    }
}

type Result<T> = std::result::Result<T, PermissionError>;

impl LessonPermission {
    pub async fn of_entity(
        db: &PgPool,
        account_id: AccountID,
        lesson_id: LessonID,
    ) -> Result<LessonPermission> {
        LessonPermission::type_of_entity(db, &account_id, &lesson_id)
            .await
            .map(|permission_type| LessonPermission {
                permission_type,
                account_id,
                lesson_id,
            })
    }

    pub async fn type_of_entity(
        db: &PgPool,
        account_id: &AccountID,
        lesson_id: &LessonID,
    ) -> Result<PermissionType> {
        let mut transaction = db.begin().await?;
        let (entity_exists,): (bool,) =
            sqlx::query_as("SELECT EXISTS (SELECT FROM Lesson WHERE id = $1)")
                .bind(lesson_id)
                .fetch_one(&mut transaction)
                .await?;

        if !entity_exists {
            return Err(PermissionError::LessonNotPresent);
        }

        let res: Option<(PgPermissionType,)> = sqlx::query_as(
            "SELECT type FROM LessonPermission WHERE lesson_id = $1 AND account_id = $2",
        )
        .bind(&lesson_id)
        .bind(&account_id)
        .fetch_optional(db)
        .await?;

        res.ok_or(PermissionError::PermissionNotPresent)
            .map(|(permission_type,)| permission_type.into())
    }

    pub(crate) async fn save_in_transaction(
        transaction: &mut Transaction,
        permission_type: PermissionType,
        lesson_id: &LessonID,
        account_id: &AccountID,
    ) -> sqlx::Result<()> {
        let permission_type: PgPermissionType = permission_type.into();
        sqlx::query(
            "INSERT INTO LessonPermission (type, lesson_id, account_id) VALUES ($1, $2, $3)",
        )
        .bind(permission_type)
        .bind(lesson_id)
        .bind(account_id)
        .execute(transaction)
        .await
        .map(|_| ())
    }
}
