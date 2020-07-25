use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgQueryAs};
use listenfd::ListenFd;
use std::env;

#[derive(Deserialize)]
struct GreetInfo {
    id: u32,
    name: String,
}

#[get("/{id}/{name}/hello")]
async fn greet(info: web::Path<GreetInfo>) -> impl Responder {
    format!("Hello {}, id: {}", info.name, info.id)
}

#[derive(Serialize, Deserialize)]
struct MyObj {
    f: i32,
    name: String,
}

#[get("/json")]
async fn get_json() -> HttpResponse {
    HttpResponse::Ok().json(MyObj {
        f: 4,
        name: "hello".to_string(),
    })
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct TestData {
    id: i32,
    name: String,
}

#[get("/data")]
async fn get_data(db: web::Data<PgPool>) -> HttpResponse {
    // let data = sqlx::query_as::<_, TestData>("SELECT * FROM test");
    // let data = sqlx::query!("SELECT id, name FROM test")
    //     .fetch_all(db.get_ref())
    //     .await;
    let maybe_data = sqlx::query_as::<_, TestData>("SELECT id, name FROM test")
        .fetch_all(db.get_ref())
        .await;
    // let maybe_data = sqlx::query_as!(TestData, "SELECT id, name FROM test")
    //     .fetch_all(db.get_ref())
    //     .await;

    let data = match maybe_data {
        Ok(data) => data,
        Err(_) => return HttpResponse::Ok().json("Err")
    };

    HttpResponse::Ok().json(data)
}

#[actix_rt::main]
async fn main() -> Result<()> {

    let mut listenfd = ListenFd::from_env();

    let pool = PgPool::new("postgresql://postgres:postgres@localhost:5433/nestudent").await?;

    let mut server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(greet)
            .service(get_json)
            .service(get_data)
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
