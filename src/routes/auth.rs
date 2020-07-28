use actix_web::{post, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::APIError;
use crate::model::account::get_account_by_login;
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
