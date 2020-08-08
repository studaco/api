pub mod auth;
pub mod lesson;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    auth::configure_auth_routes(cfg);
    lesson::configure_lesson_routes(cfg);
}