pub mod auth;
pub mod lesson;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(auth::login)
        .service(auth::register)
        .service(lesson::get_lesson)
        .service(lesson::patch_lesson)
        .service(lesson::put_lesson)
        .service(lesson::delete_lesson);
}