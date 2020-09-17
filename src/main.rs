use actix_web::{App, HttpServer, middleware::{Compress, Logger, NormalizePath}, web};
use anyhow::Result;
use dotenv::dotenv;
use include_dir::{include_dir, Dir};
use listenfd::ListenFd;
use sqlx::postgres::PgPool;
use sqlx_pg_migrate::migrate;
use std::env;
use env_logger;
use actix_rt::time::delay_for;
use std::time::Duration;

mod error;
mod model;
mod payload;
mod routes;
mod token;
mod util;
mod middleware;
use routes::configure_routes;

static MIGRATIONS: Dir = include_dir!("sql");

async fn wait_for_db<T, E, F, Op>(op: Op) -> T
where 
    F: std::future::Future<Output = Result<T, E>>,
    E: std::error::Error,
    Op: Fn() -> F
{
    let timeout: u64 = std::env::var("DB_TIMEOUT")
        .map_or_else(|_| 10, |value| value.parse().unwrap());
    loop {
        let res = op().await;
        match res {
            Ok(result) => break result,
            Err(error) => {
                log::error!("Cannot connect to the database ({}). Trying again in {} seconds", error, timeout);
                delay_for(Duration::from_secs(timeout)).await;
            }
        };
    }
}

#[actix_rt::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let mut listenfd = ListenFd::from_env();

    let db_url: String =
        env::var("DATABASE_URL").expect("DATABASE_URL variable is not set properly");

    wait_for_db(|| migrate(&db_url, &MIGRATIONS)).await;

    let pool = wait_for_db(|| PgPool::new(&db_url)).await;

    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(web::JsonConfig::default().error_handler(error::json_error_handler))
            .app_data(web::PathConfig::default().error_handler(error::path_error_handler))
            .app_data(web::QueryConfig::default().error_handler(error::query_error_handler))
            .data(pool.clone())
            .wrap(Compress::default())
            .wrap(NormalizePath)
            .wrap(Logger::default())
            // TODO: Add CORS support
            .configure(configure_routes)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host = env::var("HOST").expect("HOST variable is not set in environment");
            let port = env::var("PORT").expect("PORT variable is not set in environment");
            let addr = format!("{}:{}", host, port);
            println!("Server is listening on {}", addr);
            server.bind(format!("{}:{}", host, port))?
        }
    };

    server.run().await?;
    Ok(())
}
