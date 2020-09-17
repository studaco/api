use actix_http::Payload;
use actix_web::{FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{PgPool, PgQueryAs, PgRow},
    Row,
};

use crate::model::{
    account::AccountID, 
    teacher::TeacherID,
    Transaction,
    permission::{PermissionType, PgPermissionType, PermissionError, Result}
};
use crate::error::APIError;

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct TeacherPermission {
    pub permission_type: PermissionType,
    pub teacher_id: TeacherID,
    pub account_id: AccountID,
}

impl FromRequest for TeacherPermission {
    type Error = APIError;
    type Future = Ready<std::result::Result<TeacherPermission, APIError>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(
            req.extensions()
                .get::<TeacherPermission>()
                .map(|id| id.clone())
                .ok_or(APIError::InternalError {
                    message: "Error encountered while processing permission checks"
                        .to_string(),
                }),
        )
    }
}

// How to make this one generic over database types?
impl<'c> sqlx::FromRow<'c, PgRow<'c>> for TeacherPermission {
    fn from_row(row: &PgRow<'c>) -> sqlx::Result<Self> {
        let permission_type: PgPermissionType = row.try_get("type")?;
        let teacher_id: TeacherID = row.try_get("teacher_id")?;
        let account_id: AccountID = row.try_get("account_id")?;

        Ok(TeacherPermission {
            permission_type: permission_type.into(),
            teacher_id,
            account_id,
        })
    }
}

impl TeacherPermission {
    pub async fn of_entity(
        db: &PgPool,
        account_id: AccountID,
        teacher_id: TeacherID,
    ) -> Result<TeacherPermission> {
        TeacherPermission::type_of_entity(db, &account_id, &teacher_id)
            .await
            .map(|permission_type| TeacherPermission {
                permission_type,
                account_id,
                teacher_id: teacher_id,
            })
    }

    pub async fn type_of_entity(
        db: &PgPool,
        account_id: &AccountID,
        teacher_id: &TeacherID,
    ) -> Result<PermissionType> {
        let mut transaction = db.begin().await?;
        let (entity_exists,): (bool,) =
            sqlx::query_as("SELECT EXISTS (SELECT FROM Teacher WHERE id = $1)")
                .bind(teacher_id)
                .fetch_one(&mut transaction)
                .await?;

        if !entity_exists {
            return Err(PermissionError::EntityNotPresent);
        }

        let res: Option<(PgPermissionType,)> = sqlx::query_as(
            "SELECT type FROM TeacherPermission WHERE teacher_id = $1 AND account_id = $2",
        )
        .bind(&teacher_id)
        .bind(&account_id)
        .fetch_optional(db)
        .await?;

        res.ok_or(PermissionError::PermissionNotPresent)
            .map(|(permission_type,)| permission_type.into())
    }

    pub async fn save_in_transaction(
        transaction: &mut Transaction,
        permission_type: PermissionType,
        teacher_id: &TeacherID,
        account_id: &AccountID,
    ) -> sqlx::Result<()> {
        let permission_type: PgPermissionType = permission_type.into();
        sqlx::query(
            "INSERT INTO TeacherPermission (type, teacher_id, account_id) VALUES ($1, $2, $3)",
        )
        .bind(permission_type)
        .bind(teacher_id)
        .bind(account_id)
        .execute(transaction)
        .await
        .map(|_| ())
    }
}
