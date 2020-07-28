use actix_http::{Error, Response};
use actix_web::{http::StatusCode, HttpRequest, Responder};
use futures::future::{ok, Ready};
use serde::Serialize;

#[derive(Serialize)]
pub struct Payload<T>
where
    T: Serialize,
{
    payload: T,
}

impl<T> Responder for Payload<T>
where
    T: Serialize,
{
    type Error = Error;
    type Future = Ready<Result<Response, Error>>;

    fn respond_to(self, _: &HttpRequest) -> Self::Future {
        ok(Response::build(StatusCode::OK).json(self))
    }
}

impl <T> From<T> for Payload<T> where T: Serialize {
    fn from(payload: T) -> Self {
        Payload { payload }
    }
}