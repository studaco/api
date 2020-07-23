#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use std::vec;
use serde::Serialize;
use rocket_contrib::json::Json;

#[get("/")]
fn index() -> &'static str {
    "Hello from the outside"
}

#[get("/greet/<user>/<id>")]
fn greet(user: String, id: i32) -> String {
    format!("Hello {}, id: {}", user, id)
}

#[derive(Debug, Serialize)]
struct SomeData<'a> {
    f: i32,
    name: &'a str,
}

#[get("/json")]
fn get_json() -> Json<SomeData<'static>> {
    Json(SomeData {f: 5, name: "hello" })
}

fn main() {
    rocket::ignite().mount("/", routes![index, greet, get_json]).launch();
}
