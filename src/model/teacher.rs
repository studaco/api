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
use super::permission::{LessonPermission, PermissionType};
use super::repeat::Repeat;
use super::single_occurrence::SingleOccurrence;
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
    async fn by_id(db: &PgPool, teacher_id: &TeacherID) -> sqlx::Result<Option<Teacher>> {
        sqlx::query_as("SELECT id, first_name, last_name, user_id FROM Teacher WHERE id = $1")
            .bind(teacher_id)
            .fetch_optional(db)
            .await
    }
}
