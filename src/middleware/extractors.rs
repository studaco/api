use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use serde::de::DeserializeOwned;
use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, web, Error, FromRequest};
use futures::future::{err, ok, Ready};
use futures::Future;

pub struct PathExtractor<T>
where
    T: DeserializeOwned,
{
    marker: PhantomData<T>,
}

impl<T: DeserializeOwned + 'static> Default for PathExtractor<T> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<T: DeserializeOwned + 'static> PathExtractor<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T, S, B> Transform<S> for PathExtractor<T>
where
    T: DeserializeOwned + 'static,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = PathExtractorMiddleware<T, S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(PathExtractorMiddleware {
            service,
            marker: PhantomData,
        })
    }
}

pub struct PathExtractorMiddleware<T, S> {
    marker: PhantomData<T>,
    service: S,
}

impl<T, S, B> Service for PathExtractorMiddleware<T, S>
where
    T: DeserializeOwned + 'static,
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

        let res = web::Path::<T>::from_request(&http_req, &mut payload)
            .into_inner()
            .map(|path| path.into_inner());

        match res {
            Ok(path_data) => {
                http_req.extensions_mut().insert(path_data);
                let new_req =
                    ServiceRequest::from_parts(http_req, payload).unwrap_or_else(|_| panic!("???"));
                Box::pin(self.service.call(new_req))
            }
            Err(error) => Box::pin(err(Error::from(error))),
        }
    }
}
