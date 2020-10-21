use actix_web::{post, web, HttpResponse};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::{APIError, Result};
use crate::middleware::Authentication;
use crate::model::account::{Account, AccountID};
use crate::token::{
    generate_token_pair, AccessToken, AccessTokenInfo, ApplicationClaim, ApplicationToken,
    RefreshToken,
};
use crate::types::RedisPool;

#[derive(Deserialize)]
pub struct LoginData {
    login: String,
    password: String,
}

#[derive(Serialize)]
pub struct AuthorizationResponse {
    access_token: AccessToken,
    refresh_token: RefreshToken,
}

#[post("/auth/login")]
pub async fn login(
    db: web::Data<PgPool>,
    login_data: web::Json<LoginData>,
) -> Result<AuthorizationResponse> {
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

    let (access_token, refresh_token) = generate_token_pair(account.id)?;

    Ok(AuthorizationResponse {
        access_token,
        refresh_token,
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
) -> Result<AuthorizationResponse> {
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

    let (access_token, refresh_token) = generate_token_pair(account.id)?;

    Ok(AuthorizationResponse {
        access_token,
        refresh_token,
    }
    .into())
}

#[post("/auth/revoke", wrap = "Authentication")]
pub async fn revoke(
    redis_pool: web::Data<RedisPool>,
    account_id: AccountID,
    claim: ApplicationClaim<AccessTokenInfo>,
) -> std::result::Result<HttpResponse, APIError> {
    let mut redis = redis_pool.get().await?;

    let key = format!("revoked_token:{}:{}", account_id, claim.inner.token_origin);
    let now = chrono::Utc::now().timestamp();
    let ttl = claim.registered.expiration - now;
    if ttl > 0 {
        redis.set_ex(key, 1_u8, ttl as usize).await?;
    }

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    refresh_token: RefreshToken,
}

#[post("/auth/refresh")]
pub async fn refresh(
    redis_pool: web::Data<RedisPool>,
    request: web::Json<RefreshTokenRequest>,
) -> Result<AuthorizationResponse> {
    let mut redis = redis_pool.get().await?;

    let RefreshTokenRequest { refresh_token } = request.into_inner();
    let claim = refresh_token.authenticate_claim()?;
    let token_key = format!(
        "revoked_token:{}:{}",
        claim.inner.account_id, claim.inner.token_id
    );
    let revoked: Option<u8> = redis.get(token_key).await?;
    if let Some(1) = revoked {
        return Err(APIError::TokenRevoked);
    }

    let user_key = format!("deleted_user:{}", claim.inner.account_id);
    let revoked_user: Option<u8> = redis.get(user_key).await?;
    if let Some(1) = revoked_user {
        return Err(APIError::TokenRevoked);
    }

    let (access_token, refresh_token) = generate_token_pair(claim.inner.account_id)?;

    Ok(AuthorizationResponse {
        access_token,
        refresh_token,
    }
    .into())
}

// TODO: delete user

pub fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(login)
        .service(register)
        .service(revoke)
        .service(refresh);
}
