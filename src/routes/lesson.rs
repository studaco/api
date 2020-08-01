use actix_web::{get, web, HttpRequest};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::APIError;
use crate::model::lesson::Lesson;
use crate::payload::Payload;
use crate::token::authorize_headers;

#[get("/lesson/{id}")]
pub async fn get_lesson(
    db: web::Data<PgPool>,
    lesson_id: web::Path<Uuid>,
    request: HttpRequest,
) -> Result<Payload<Lesson>, APIError> {
    let id = authorize_headers(request.headers())?;
    Ok(Lesson::of_user(db.get_ref(), &id, lesson_id.into_inner())
        .await?
        .into())
}
