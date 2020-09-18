use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, web, Error, FromRequest};
use futures::future::{ok, Ready};
use futures::Future;
use sqlx::PgPool;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::error::APIError;
use crate::model::account::AccountID;
use crate::model::permission::{EntityPermission, PermissionType};

#[derive(Debug, Copy, Clone)]
pub struct CheckPermission<T> {
    permission_type: Option<PermissionType>,
    marker: PhantomData<T>,
}

impl<T> Default for CheckPermission<T>
where
    T: EntityPermission,
{
    fn default() -> CheckPermission<T> {
        CheckPermission {
            permission_type: None,
            marker: PhantomData,
        }
    }
}

impl<T> CheckPermission<T>
where
    T: EntityPermission,
{
    pub fn new(permission_type: PermissionType) -> CheckPermission<T> {
        CheckPermission {
            permission_type: Some(permission_type),
            marker: PhantomData,
        }
    }
}

impl<S, B, T> Transform<S> for CheckPermission<T>
where
    T: EntityPermission + 'static,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckPermissionMiddleware<S, T>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CheckPermissionMiddleware {
            service: Rc::new(RefCell::new(service)),
            permission_type: self.permission_type,
            marker: PhantomData,
        })
    }
}

pub struct CheckPermissionMiddleware<S, T> {
    service: Rc<RefCell<S>>,
    permission_type: Option<PermissionType>,
    marker: PhantomData<T>,
}

impl<S, B, T> Service for CheckPermissionMiddleware<S, T>
where
    T: EntityPermission + 'static,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
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

        let mut service = self.service.clone();
        let expected_permission = self.permission_type.clone();

        Box::pin(async move {
            let db = web::Data::<PgPool>::from_request(&http_req, &mut payload).into_inner()?;
            let account_id = AccountID::from_request(&http_req, &mut payload).into_inner()?;
            let entity_id = <T::EntityID>::from_request(&http_req, &mut payload).await?;

            let permission: T = <T>::of_entity(db.get_ref(), account_id, entity_id)
                .await
                .map_err(|error| Error::from(APIError::from(error)))?;

            if let (Some(PermissionType::ReadWrite), PermissionType::Read) =
                (expected_permission, permission.permission())
            {
                return Err(APIError::NoWriteAccess.into());
            }

            http_req.extensions_mut().insert(permission);
            let new_req =
                ServiceRequest::from_parts(http_req, payload).unwrap_or_else(|_| panic!("???"));

            service.call(new_req).await
        })
    }
}
