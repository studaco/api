use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::model::account::{get_account_by_id, register_account};

#[get("/test/{id}")]
pub async fn test(db: web::Data<PgPool>, id: web::Path<Uuid>) -> HttpResponse {
    let account = get_account_by_id(db.get_ref(), id.into_inner()).await;

    match account {
        Ok(account) => match account {
            Some(account) => HttpResponse::Ok().json(account),
            None => HttpResponse::NotFound().body("User does not exist"),
        },
        Err(err) => HttpResponse::InternalServerError().body(format!("{}", err)),
    }
}

#[derive(Deserialize)]
// #[serde(rename_all = "camelCase")]
pub struct RegistrationInfo {
    first_name: String,
    last_name: Option<String>,
    login: String,
    password: String,
}

#[post("/test/register")]
pub async fn register(
    db: web::Data<PgPool>,
    registration: web::Json<RegistrationInfo>,
) -> HttpResponse {
    let RegistrationInfo {
        first_name,
        last_name,
        login,
        password,
    } = registration.into_inner();
    let result = register_account(db.get_ref(), first_name, last_name, login, password).await;

    match result {
        Ok(account) => HttpResponse::Ok().json(account),
        Err(err) => HttpResponse::InternalServerError().body(format!("{}", err)),
    }
}
