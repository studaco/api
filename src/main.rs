use actix_web::{App, HttpServer};
use anyhow::Result;
use dotenv::dotenv;
use include_dir::{include_dir, Dir};
use listenfd::ListenFd;
use sqlx::postgres::PgPool;
use sqlx_pg_migrate::migrate;
use std::env;

mod error;
pub mod model;
mod payload;
mod routes;
mod token;
use routes::*;

static MIGRATIONS: Dir = include_dir!("sql");

#[actix_rt::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let mut listenfd = ListenFd::from_env();

    let db_url: String =
        env::var("DATABASE_URL").expect("DATABASE_URL variable is not set properly");

    migrate(&db_url, &MIGRATIONS).await?;

    let pool = PgPool::new(&db_url).await?;

    let mut server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(auth::login)
            .service(auth::register)
            .service(lesson::get_lesson)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host = env::var("HOST").expect("HOST is not set in .env file");
            let port = env::var("PORT").expect("PORT is not set in .env file");
            let addr = format!("{}:{}", host, port);
            println!("Server is listening on {}", addr);
            server.bind(format!("{}:{}", host, port))?
        }
    };

    server.run().await?;
    Ok(())
}
