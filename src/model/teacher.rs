use actix_http::Payload;
use actix_web::{FromRequest, HttpRequest};
use chrono::NaiveDate;
use futures::future::{ready, Ready};
use indoc::indoc;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgQueryAs, Postgres};
use std::vec::Vec;
use uuid::Uuid;

use super::account::AccountID;
use super::permission::{PermissionType, TeacherPermission};
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
    last_name: Option<String>,
    user_id: Option<AccountID>,
}

impl Teacher {
    pub async fn by_id(db: &PgPool, teacher_id: &TeacherID) -> sqlx::Result<Option<Teacher>> {
        sqlx::query_as("SELECT id, first_name, last_name, user_id FROM Teacher WHERE id = $1")
            .bind(teacher_id)
            .fetch_optional(db)
            .await
    }

    pub async fn create(
        db: &PgPool,
        first_name: String,
        last_name: Option<String>,
        user_id: Option<AccountID>,
        owner: AccountID,
    ) -> sqlx::Result<Teacher> {
        let mut transaction = db.begin().await?;

        let (id,): (TeacherID,) = sqlx::query_as(
            "INSERT INTO Teacher (first_name, last_name, user_id) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(&first_name)
        .bind(&last_name)
        .bind(&user_id)
        .fetch_one(&mut transaction)
        .await?;

        TeacherPermission::save_in_transaction(
            &mut transaction,
            PermissionType::ReadWrite,
            &id,
            &owner,
        )
        .await?;

        Ok(Teacher {
            id,
            first_name,
            last_name,
            user_id,
        })
    }

    pub async fn update(
        db: &PgPool,
        first_name: Option<String>,
        last_name: Option<Option<String>>,
        user_id: Option<Option<AccountID>>,
    ) -> sqlx::Result<()> {
        if let (None, None, None) = (&first_name, &last_name, &user_id) {
            return Ok(());
        };
        let mut sql = "UPDATE Teacher SET ".to_string();

        if first_name.is_some() {
            sql.push_str("first_name = $1")
        }

        if last_name.is_some() {
            sql.push_str("last_name = $2");
        }

        if user_id.is_some() {
            sql.push_str("user_id = $3");
        }

        let mut query = sqlx::query(&sql[..]);

        if let Some(first_name) = first_name {
            query = query.bind(first_name);
        }

        if let Some(last_name) = last_name {
            query = query.bind(last_name);
        }

        if let Some(user_id) = user_id {
            query = query.bind(user_id);
        }

        query.execute(db).await.map(|_| ())
    }

    pub async fn delete(db: &PgPool, teacher_id: &TeacherID) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM Teacher WHERE id = $1")
            .bind(teacher_id)
            .execute(db)
            .await
            .map(|_| ())
    }
}
