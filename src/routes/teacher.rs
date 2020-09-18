use actix_web::{delete, get, patch, put, web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::{APIError, Result};
use crate::middleware::{Authentication, CheckPermission, PathExtractor};
use crate::model::{
    account::AccountID,
    permission::{PermissionType, TeacherPermission},
    teacher::{Teacher, TeacherID},
};
use crate::payload::Payload;
use crate::util::deserialize_optional_field;

#[get(
    "/teacher/{id}",
    wrap = "CheckPermission::<TeacherPermission>::new(PermissionType::Read)",
    wrap = "PathExtractor::<TeacherID>::new()",
    wrap = "Authentication"
)]
pub async fn get_teacher(db: web::Data<PgPool>, teacher_id: TeacherID) -> Result<Teacher> {
    Teacher::by_id(db.get_ref(), teacher_id)
        .await?
        .ok_or(APIError::LessonDosNotExist)
        .map(Payload::from)
}

#[derive(Deserialize)]
pub struct TeacherCreateRequest {
    first_name: String,
    last_name: Option<String>,
}

#[put("/teacher", wrap = "Authentication")]
pub async fn put_teacher(
    db: web::Data<PgPool>,
    lesson: web::Json<TeacherCreateRequest>,
    account_id: AccountID,
) -> Result<Teacher> {
    let TeacherCreateRequest {
        first_name,
        last_name,
    } = lesson.into_inner();

    Ok(Teacher::create(
        db.get_ref(),
        first_name,
        last_name,
        None, // TODO: add user affiliation
        &account_id,
    )
    .await?
    .into())
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct TeacherUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    first_name: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_field")]
    #[serde(skip_serializing_if = "Option::is_none")]
    last_name: Option<Option<String>>,
}

#[patch(
    "/teacher/{id}",
    wrap = "CheckPermission::<TeacherPermission>::new(PermissionType::ReadWrite)",
    wrap = "PathExtractor::<TeacherID>::new()",
    wrap = "Authentication"
)]
pub async fn patch_teacher(
    db: web::Data<PgPool>,
    teacher_id: TeacherID,
    patch: web::Json<TeacherUpdateRequest>,
) -> std::result::Result<HttpResponse, APIError> {
    let TeacherUpdateRequest {
        first_name,
        last_name,
    } = patch.into_inner();

    Teacher::update(db.get_ref(), &teacher_id, first_name, last_name, None).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[delete(
    "/teacher/{id}",
    wrap = "CheckPermission::<TeacherPermission>::new(PermissionType::ReadWrite)",
    wrap = "PathExtractor::<TeacherID>::new()",
    wrap = "Authentication"
)]
pub async fn delete_teacher(
    db: web::Data<PgPool>,
    teacher_id: TeacherID,
) -> std::result::Result<HttpResponse, APIError> {
    Teacher::delete(db.get_ref(), &teacher_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("/teachers", wrap = "Authentication")]
pub async fn get_teachers(db: web::Data<PgPool>, account_id: AccountID) -> Result<Vec<Teacher>> {
    Ok(Teacher::of_user(db.get_ref(), &account_id).await?.into())
}

pub fn configure_teacher_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_teacher)
        .service(put_teacher)
        .service(patch_teacher)
        .service(delete_teacher)
        .service(get_teachers);
}
