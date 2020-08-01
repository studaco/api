use actix_web::{web, HttpResponse};
use serde::{Deserialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::APIError;
use crate::token::{authorize_headers};
use crate::model::permission::PermissionType;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Teacher,
    Lesson,
    Task,
}

#[derive(Deserialize)]
pub struct PermissionRequest {
    enitity_type: EntityType,
    entity_id: Uuid,
    #[serde(rename = "type")]
    permission_type: PermissionType,
    user_id: Uuid,
}

pub async fn add_permission(
    db: web::Data<PgPool>,
    request: web::HttpRequest,
    permission: web::Json<PermissionRequest>,
) -> Result<HttpResponse, APIError> {
    let owner_id = authorize_headers(request.headers())?;

    // let PermissionRequest { enitity_type, entity_id, permission_type } = permission.into_inner();

    Ok(HttpResponse::Created().finish())
}
