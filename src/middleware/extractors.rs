use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, web, Error, FromRequest};
use futures::future::{err, ok, Ready};
use futures::Future;

use crate::model::lesson::LessonID;

pub struct ExtractLessonID;

impl<S, B> Transform<S> for ExtractLessonID
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ExtractLessonIDMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ExtractLessonIDMiddleware { service })
    }
}

pub struct ExtractLessonIDMiddleware<S> {
    service: S,
}

impl<S, B> Service for ExtractLessonIDMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let (http_req, mut payload) = req.into_parts();

        let res = web::Path::<LessonID>::from_request(&http_req, &mut payload)
            .into_inner()
            .map(|path| path.into_inner());

        match res {
            Ok(lesson_id) => {
                http_req.extensions_mut().insert(lesson_id);
                let new_req =
                    ServiceRequest::from_parts(http_req, payload).unwrap_or_else(|_| panic!("???"));
                Box::pin(self.service.call(new_req))
            }
            Err(error) => Box::pin(err(Error::from(error))),
        }
    }
}
