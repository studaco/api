use actix_http::Payload;
use actix_web::{FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use indoc::indoc;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgQueryAs};
use uuid::Uuid;

use super::account::AccountID;
use super::permission::{PermissionType, TeacherPermission, EntityPermission};
use crate::error::APIError;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct TeacherID(Uuid);

impl FromRequest for TeacherID {
    type Error = APIError;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(
            req.extensions()
                .get::<TeacherID>()
                .map(|id| id.clone())
                .ok_or(APIError::InternalError {
                    message: "Error encountered while extracting parameters".to_string(),
                }),
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct Teacher {
    id: TeacherID,
    first_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_name: Option<String>,
    #[sqlx(rename = "account_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    associated_account_id: Option<AccountID>,
}

impl Teacher {
    pub async fn by_id(db: &PgPool, teacher_id: TeacherID) -> sqlx::Result<Option<Teacher>> {
        sqlx::query_as("SELECT id, first_name, last_name, account_id FROM Teacher WHERE id = $1")
            .bind(teacher_id)
            .fetch_optional(db)
            .await
    }

    pub async fn create(
        db: &PgPool,
        first_name: String,
        last_name: Option<String>,
        associated_account_id: Option<AccountID>,
        owner: &AccountID,
    ) -> sqlx::Result<Teacher> {
        let mut transaction = db.begin().await?;

        let (id,): (TeacherID,) = sqlx::query_as(
            "INSERT INTO Teacher (first_name, last_name, account_id) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(&first_name)
        .bind(&last_name)
        .bind(&associated_account_id)
        .fetch_one(&mut transaction)
        .await?;

        TeacherPermission::save_in_transaction(
            &mut transaction,
            PermissionType::ReadWrite,
            &id,
            owner,
        )
        .await?;

        transaction.commit().await?;

        Ok(Teacher {
            id,
            first_name,
            last_name,
            associated_account_id,
        })
    }

    pub async fn update(
        db: &PgPool,
        teacher_id: &TeacherID,
        first_name: Option<String>,
        last_name: Option<Option<String>>,
        associated_account_id: Option<Option<AccountID>>,
    ) -> sqlx::Result<()> {
        let mut parts = Vec::<String>::with_capacity(3);
        let mut counter = 0_u8;

        if first_name.is_some() {
            counter += 1;
            parts.push(format!("first_name = ${}", counter));
        }

        if last_name.is_some() {
            counter += 1;
            parts.push(format!("last_name = ${}", counter));
        }

        if associated_account_id.is_some() {
            counter += 1;
            parts.push(format!("account_id = ${}", counter));
        }

        if counter == 0 {
            return Ok(());
        }

        let sql = format!(
            "UPDATE Teacher SET {} WHERE id = ${}",
            parts.join(","),
            counter + 1
        );
        let mut query = sqlx::query(&sql[..]);

        if let Some(first_name) = first_name {
            query = query.bind(first_name);
        }

        if let Some(last_name) = last_name {
            query = query.bind(last_name);
        }

        if let Some(user_id) = associated_account_id {
            query = query.bind(user_id);
        }

        query.bind(teacher_id).execute(db).await.map(|_| ())
    }

    pub async fn delete(db: &PgPool, teacher_id: &TeacherID) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM Teacher WHERE id = $1")
            .bind(teacher_id)
            .execute(db)
            .await
            .map(|_| ())
    }

    pub async fn of_user(db: &PgPool, account_id: &AccountID) -> sqlx::Result<Vec<Teacher>> {
        sqlx::query_as(indoc! {"
            SELECT id, first_name, last_name, Teacher.account_id
            FROM Teacher
            JOIN TeacherPermission ON Teacher.id = TeacherPermission.teacher_id
            WHERE TeacherPermission.account_id = $1
        "})
        .bind(account_id)
        .fetch_all(db)
        .await
    }
}
