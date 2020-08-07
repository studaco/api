use actix_web::{get, put, web, HttpRequest};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use std::vec::Vec;

use crate::error::{APIError, Result};
use crate::model::lesson::{Lesson, Repeat};
use crate::model::permission::{LessonPermission};
use crate::token::authorize_headers;

#[get("/lesson/{id}")]
pub async fn get_lesson(
    db: web::Data<PgPool>,
    lesson_id: web::Path<Uuid>,
    request: HttpRequest,
) -> Result<Lesson> {
    let id = authorize_headers(request.headers())?;

    let lesson_id = lesson_id.into_inner();

    LessonPermission::type_of_entity(db.get_ref(), &id, &lesson_id)
        .await?
        .ok_or(APIError::NoReadAccess)?;

    let lesson = Lesson::of_user(db.get_ref(), lesson_id).await?;

    Ok(lesson.ok_or(APIError::LessonDosNotExist)?.into())
}

#[derive(Deserialize)]
pub struct LessonCreateRequest {
    title: String,
    description: Option<String>,
    repeats: Vec<Repeat>,
}

#[put("/lesson")]
pub async fn put_lesson(
    db: web::Data<PgPool>,
    lesson: web::Json<LessonCreateRequest>,
    request: HttpRequest
) -> Result<Lesson> {
    let account_id = authorize_headers(request.headers())?;
    let LessonCreateRequest { title, description, repeats } = lesson.into_inner();
    Ok(Lesson::create(db.get_ref(), title, description, repeats, &account_id).await?.into())
}

#[patch("/lesson/{id}")]
pub async fn patch_lesson(
    db: web::Data<PgPool>,
    id: web::Path<Uuid>,
    patch: web::Json<LessonCreateRequest>,
    request: HttpRequest
) -> Result<HttpResponse>