use actix_web::FromRequest;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;

use crate::error::APIError;
use crate::model::{account::AccountID, lesson::LessonID, teacher::TeacherID, Transaction};

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

#[async_trait]
pub trait EntityPermission: Sized + FromRequest {
    type EntityID;

    async fn of_entity(
        db: &PgPool,
        account_id: AccountID,
        entity_id: Self::EntityID,
    ) -> Result<Self>;

    async fn type_of_entity(
        db: &PgPool,
        account_id: &AccountID,
        entity_id: &Self::EntityID,
    ) -> Result<PermissionType>;

    async fn save_in_transaction(
        transaction: &mut Transaction,
        permission_type: PermissionType,
        entity_id: &Self::EntityID,
        account_id: &AccountID,
    ) -> sqlx::Result<()>;
}

#[macro_export]
macro_rules! impl_permission {
    ($type:ty, $id_type:ty, $column_name:expr, $entity_type_table:expr, $entity_permission_table:expr) => {
        impl actix_web::FromRequest for $type {
            type Error = crate::error::APIError;
            type Future =
                futures::future::Ready<std::result::Result<$type, crate::error::APIError>>;
            type Config = ();

            fn from_request(
                req: &actix_web::HttpRequest,
                _: &mut actix_http::Payload,
            ) -> Self::Future {
                futures::future::ready(req.extensions().get::<$type>().map(|id| id.clone()).ok_or(
                    crate::error::APIError::InternalError {
                        message: "Error encountered while processing permission checks".to_string(),
                    },
                ))
            }
        }

        impl<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow<'c>> for $type {
            fn from_row(row: &sqlx::postgres::PgRow<'c>) -> sqlx::Result<Self> {
                let permission_type: crate::model::permission::PgPermissionType =
                    sqlx::Row::try_get(row, "type")?;
                let entity_id: $id_type = sqlx::Row::try_get(row, $column_name)?;
                let account_id: AccountID = sqlx::Row::try_get(row, "account_id")?;
                let permission: PermissionType = permission_type.into();

                Ok(<$type>::new(permission, entity_id, account_id))
            }
        }

        #[async_trait::async_trait]
        impl crate::model::permission::EntityPermission for $type {
            type EntityID = $id_type;

            async fn of_entity(
                db: &sqlx::PgPool,
                account_id: crate::model::account::AccountID,
                entity_id: Self::EntityID,
            ) -> crate::model::permission::Result<Self> {
                Self::type_of_entity(db, &account_id, &entity_id)
                    .await
                    .map(|permission_type| <$type>::new(permission_type, entity_id, account_id))
            }

            async fn type_of_entity(
                db: &sqlx::PgPool,
                account_id: &crate::model::account::AccountID,
                entity_id: &Self::EntityID,
            ) -> crate::model::permission::Result<crate::model::permission::PermissionType> {
                use sqlx::postgres::PgQueryAs;
                let mut transaction = db.begin().await?;
                let (entity_exists,): (bool,) = sqlx::query_as(concat!(
                    "SELECT EXISTS (SELECT FROM ",
                    $entity_type_table,
                    " WHERE id = $1)"
                ))
                .bind(entity_id)
                .fetch_one(&mut transaction)
                .await?;

                if !entity_exists {
                    return Err(crate::model::permission::PermissionError::EntityNotPresent);
                }

                let res: Option<(crate::model::permission::PgPermissionType,)> =
                    sqlx::query_as(concat!(
                        "SELECT type FROM ",
                        $entity_permission_table,
                        " WHERE ",
                        $column_name,
                        " = $1 AND account_id = $2"
                    ))
                    .bind(&entity_id)
                    .bind(&account_id)
                    .fetch_optional(db)
                    .await?;

                res.ok_or(crate::model::permission::PermissionError::PermissionNotPresent)
                    .map(|(permission_type,)| permission_type.into())
            }

            async fn save_in_transaction(
                transaction: &mut crate::model::Transaction,
                permission_type: crate::model::permission::PermissionType,
                entity_id: &Self::EntityID,
                account_id: &AccountID,
            ) -> sqlx::Result<()> {
                let permission_type: crate::model::permission::PgPermissionType =
                    permission_type.into();
                sqlx::query(concat!(
                    "INSERT INTO ",
                    $entity_permission_table,
                    " (type, ",
                    $column_name,
                    ", account_id) VALUES ($1, $2, $3)"
                ))
                .bind(permission_type)
                .bind(entity_id)
                .bind(account_id)
                .execute(transaction)
                .await
                .map(|_| ())
            }
        }
    };
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct LessonPermission {
    pub permission_type: PermissionType,
    pub lesson_id: LessonID,
    pub account_id: AccountID,
}

impl LessonPermission {
    fn new(permission_type: PermissionType, lesson_id: LessonID, account_id: AccountID) -> Self {
        Self {
            permission_type,
            lesson_id,
            account_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct TeacherPermission {
    pub permission_type: PermissionType,
    pub teacher_id: TeacherID,
    pub account_id: AccountID,
}

impl TeacherPermission {
    fn new(permission_type: PermissionType, teacher_id: TeacherID, account_id: AccountID) -> Self {
        Self {
            permission_type,
            teacher_id,
            account_id,
        }
    }
}

impl_permission!(
    LessonPermission,
    LessonID,
    "lesson_id",
    "Lesson",
    "LessonPermission"
);

impl_permission!(
    TeacherPermission,
    TeacherID,
    "teacher_id",
    "Teacher",
    "TeacherPermission"
);
