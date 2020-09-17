pub mod auth;
pub mod lesson;
pub mod teacher;

use actix_web::web;
use actix_web::{get};
use serde::Serialize;
use crate::payload::Payload;

#[derive(Serialize)]
struct ServiceInfo<'c> {
    name: &'c str,
    version: &'c str,
    author: &'c str
}

#[get("/serviceinfo")]
async fn serviceinfo() -> Payload<ServiceInfo<'static>> {
    ServiceInfo {
        name: std::env!("CARGO_PKG_NAME"),
        author: std::env!("CARGO_PKG_AUTHORS"),
        version: std::env!("CARGO_PKG_VERSION")
    }.into()
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(serviceinfo);
    auth::configure_auth_routes(cfg);
    lesson::configure_lesson_routes(cfg);
    teacher::configure_teacher_routes(cfg)
}