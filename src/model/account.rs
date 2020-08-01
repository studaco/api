use bcrypt::BcryptError;
use serde::Serialize;
use sqlx::{postgres::PgQueryAs, PgPool};
use thiserror::Error;
use uuid::Uuid;

use crate::error::APIError;

#[derive(Debug, Serialize)]
pub struct Account {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: Option<String>,
    pub login: String,
    pub password_hash: String,
}

#[derive(Error, Debug)]
pub enum RegistrationError {
    #[error("Login is no unique")]
    LoginNotUnique,
    #[error("{0}")]
    Database(#[from] sqlx::Error),
    #[error("{0}")]
    Bcrypt(#[from] BcryptError),
}

impl From<RegistrationError> for APIError {
    fn from(err: RegistrationError) -> Self {
        match err {
            RegistrationError::LoginNotUnique => APIError::LoginAlreadyPresent,
            RegistrationError::Database(error) => error.into(),
            RegistrationError::Bcrypt(error) => error.into(),
        }
    }
}

impl Account {
    pub async fn get_by_id(db: &PgPool, id: Uuid) -> sqlx::Result<Option<Account>> {
        sqlx::query_as!(
            Account,
            r#"SELECT id, first_name, last_name, login, password_hash 
        FROM Account WHERE id = $1"#,
            id
        )
        .fetch_optional(db)
        .await
    }

    pub async fn get_by_login(db: &PgPool, login: String) -> sqlx::Result<Option<Account>> {
        sqlx::query_as!(
            Account,
            r#"SELECT id, first_name, last_name, login, password_hash 
        FROM Account WHERE login = $1"#,
            login
        )
        .fetch_optional(db)
        .await
    }

    pub async fn register(
        db: &PgPool,
        first_name: String,
        last_name: Option<String>,
        login: String,
        password: String,
    ) -> Result<Account, RegistrationError> {
        let hash = bcrypt::hash(password, 8)?;

        let mut transaction = db.begin().await?;
        let (count,): (i64,) = sqlx::query_as(r#"SELECT count(*) FROM Account WHERE login = $1"#)
            .bind(&login)
            .fetch_one(&mut transaction)
            .await?;

        if count == 1 {
            return Err(RegistrationError::LoginNotUnique);
        }

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
        .fetch_one(&mut transaction)
        .await?;

        transaction.commit().await?;

        Ok(Account {
            id,
            first_name,
            last_name,
            login,
            password_hash: hash,
        })
    }
}
