use serde::{Deserialize, Serialize};
use sqlx::{
    pool::PoolConnection,
    postgres::{PgConnection, PgPool, PgQueryAs, PgRow},
    Row, Transaction,
};
use thiserror::Error;
use uuid::Uuid;

#[derive(sqlx::Type)]
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

#[derive(Deserialize, Serialize, Copy, Clone)]
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
        let permission_type: PgPermissionType = row.try_get("type")?;
        let lesson_id: Uuid = row.try_get("lesson_id")?;
        let account_id: Uuid = row.try_get("account_id")?;

        Ok(LessonPermission {
            permission_type: permission_type.into(),
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

    pub(crate) async fn get_for_entity_in_transaction(
        transaction: &mut Transaction<PoolConnection<PgConnection>>,
        account_id: &Uuid,
        lesson_id: Uuid,
    ) -> sqlx::Result<Option<LessonPermission>> {
        Ok(sqlx::query_as(
            "SELECT type FROM LessonPermission WHERE lesson_id = $1 AND account_id = $2",
        )
        .bind(&lesson_id)
        .bind(&account_id)
        .fetch_optional(transaction)
        .await?)
    }

    pub(crate) async fn save_in_transaction(
        permission_type: PermissionType,
        lesson_id: &Uuid,
        account_id: &Uuid,
        transaction: &mut Transaction<PoolConnection<PgConnection>>,
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
