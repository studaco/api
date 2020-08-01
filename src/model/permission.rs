use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{PgPool, PgQueryAs, PgRow},
    Row,
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub enum PermissionType {
    #[serde(rename = "r")]
    Read,
    #[serde(rename = "rw")]
    ReadWrite,
}

#[derive(Serialize, Deserialize)]
pub struct LessonPermission {
    permission_type: PermissionType,
    lesson_id: Uuid,
    account_id: Uuid,
}

#[derive(Debug, Error)]
#[error("Invalid Permission Type. Should be either \"r\" or \"rw\"")]
struct InvalidPermissionType {}

// How to make this one generic over database types?
impl<'c> sqlx::FromRow<'c, PgRow<'c>> for LessonPermission {
    fn from_row(row: &PgRow<'c>) -> sqlx::Result<Self> {
        let permission_type: &str = row.try_get("type")?;
        let permission_type = match permission_type {
            "r" => PermissionType::Read,
            "rw" => PermissionType::ReadWrite,
            _ => return Err(sqlx::Error::Decode(Box::new(InvalidPermissionType {}))),
        };
        let lesson_id: Uuid = row.try_get("lesson_id")?;
        let account_id: Uuid = row.try_get("account_id")?;

        Ok(LessonPermission {
            permission_type,
            lesson_id,
            account_id,
        })
    }
}

impl LessonPermission {
    pub async fn get_for_entity(
        db: &PgPool,
        account_id: Uuid,
        lesson_id: Uuid,
    ) -> sqlx::Result<Option<LessonPermission>> {
        Ok(sqlx::query_as(
            "SELECT type FROM LessonPermission WHERE lesson_id = $1 AND account_id = $2",
        )
        .bind(&lesson_id)
        .bind(&account_id)
        .fetch_optional(db)
        .await?)
    }
}
