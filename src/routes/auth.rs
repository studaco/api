use actix_web::{post, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::{APIError, Result};
use crate::model::account::Account;
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
) -> Result<AccessTokenResponse> {
    let LoginData { login, password } = login_data.into_inner();
    let account = Account::get_by_login(db.get_ref(), login).await?;

    let account = match account {
        Some(account) => account,
        None => return Err(APIError::InvalidCredentials),
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
) -> Result<AccessTokenResponse> {
    let RegistrationData {
        login: registration_login,
        password,
        first_name,
        last_name,
    } = registration_data.into_inner();

    let account = Account::register(
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

pub fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(login).service(register);
}
