use actix_web::{post, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::APIError;
use crate::model::account::{get_account_by_login, register_account};
use crate::payload::Payload;
use crate::token::generate_token;

#[derive(Deserialize)]
pub struct LoginData {
    login: String,
    password: String,
}

#[derive(Serialize)]
pub struct AccessTokenResponse {
    access_token: String,
}

#[post("/auth/login")]
pub async fn login(
    db: web::Data<PgPool>,
    login_data: web::Json<LoginData>,
) -> Result<Payload<AccessTokenResponse>, APIError> {
    let LoginData { login, password } = login_data.into_inner();
    let account = get_account_by_login(db.get_ref(), login).await?;

    let account = match account {
        Some(account) => account,
        None => return Err(APIError::UserDoesNotExist),
    };

    let verification = bcrypt::verify(password, &account.password_hash[..])?;

    if !verification {
        return Err(APIError::InvalidCredentials);
    }

    Ok(AccessTokenResponse {
        access_token: generate_token(account.id)?,
    }
    .into())
}

#[derive(Deserialize)]
pub struct RegistrationData {
    login: String,
    password: String,
    first_name: String,
    last_name: Option<String>,
}

#[post("/auth/register")]
pub async fn register(
    db: web::Data<PgPool>,
    registration_data: web::Json<RegistrationData>,
) -> Result<Payload<AccessTokenResponse>, APIError> {
    let RegistrationData {
        login: registration_login,
        password,
        first_name,
        last_name,
    } = registration_data.into_inner();

    let account = register_account(
        db.get_ref(),
        first_name,
        last_name,
        registration_login,
        password,
    )
    .await?;

    Ok(AccessTokenResponse {
        access_token: generate_token(account.id)?,
    }
    .into())
}
