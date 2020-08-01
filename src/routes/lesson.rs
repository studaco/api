use actix_web::{web, HttpResponse};
use serde::{Deserialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::APIError;
use crate::token::{authorize_headers};

pub async fn get_lesson() -> Result<

    let owner_id = authorize_headers(request.headers())?;