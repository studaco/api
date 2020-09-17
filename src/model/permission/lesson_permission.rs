use actix_http::Payload;
use actix_web::{FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{PgPool, PgQueryAs, PgRow},
    Row,
};

use crate::error::APIError;
use crate::model::{
    account::AccountID,
    lesson::LessonID,
    permission::{
        PermissionError, PermissionType, PgPermissionType, Result,
    },
    Transaction,
};

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
                    message: "Error encountered while processing permission checks".to_string(),
                }),
        )
    }
}

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
            return Err(PermissionError::EntityNotPresent);
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

    pub async fn save_in_transaction(
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
