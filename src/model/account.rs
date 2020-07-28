use anyhow;
use bcrypt;
use serde::Serialize;
use sqlx::{postgres::PgQueryAs, PgPool, Result};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Account {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: Option<String>,
    pub login: String,
    pub password_hash: String,
}

pub async fn get_account_by_id(db: &PgPool, id: Uuid) -> Result<Option<Account>> {
    sqlx::query_as!(
        Account,
        r#"SELECT id, first_name, last_name, login, password_hash 
        FROM Account WHERE id = $1"#,
        id
    )
    .fetch_optional(db)
    .await
}

pub async fn get_account_by_login(db: &PgPool, login: String) -> Result<Option<Account>> {
    sqlx::query_as!(
        Account,
        r#"SELECT id, first_name, last_name, login, password_hash 
        FROM Account WHERE login = $1"#,
        login
    )
    .fetch_optional(db)
    .await
}

pub async fn register_account(
    db: &PgPool,
    first_name: String,
    last_name: Option<String>,
    login: String,
    password: String,
) -> anyhow::Result<Account> {
    let hash = bcrypt::hash(password, 8)?;
    let (id,): (Uuid,) = sqlx::query_as(
        r#"INSERT 
        INTO Account (first_name, last_name, login, password_hash)
        VALUES ($1, $2, $3, $4) 
        RETURNING id"#,
    )
    .bind(&first_name)
    .bind(&last_name)
    .bind(&login)
    .bind(&hash)
    .fetch_one(db)
    .await?;

    Ok(Account {
        id,
        first_name,
        last_name,
        login,
        password_hash: hash,
    })
}

// 200 Ok           -> { ... }
// 400 BadRequest   -> { type: "", msg: "" }
// 400 BadRequest   -> { error: { type: "", msg: "" } }
// 200 Ok           -> { payload: { ... } }