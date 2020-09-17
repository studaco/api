use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, web, Error, FromRequest};
use futures::future::{ok, Ready};
use futures::Future;
use sqlx::PgPool;
use std::cell::RefCell;
use std::rc::Rc;

use crate::error::APIError;
use crate::model::account::AccountID;
use crate::model::teacher::TeacherID;
use crate::model::permission::{TeacherPermission, PermissionType};

#[derive(Debug, Copy, Clone)]
pub struct CheckTeacherPermission {
    permission_type: Option<PermissionType>,
}

impl Default for CheckTeacherPermission {
    fn default() -> CheckTeacherPermission {
        CheckTeacherPermission {
            permission_type: None,
        }
    }
}

impl CheckTeacherPermission {
    pub fn new(permission_type: PermissionType) -> CheckTeacherPermission {
        CheckTeacherPermission {
            permission_type: Some(permission_type),
        }
    }
}

impl<S, B> Transform<S> for CheckTeacherPermission
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckTeacherPermissionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CheckTeacherPermissionMiddleware {
            service: Rc::new(RefCell::new(service)),
            permission_type: self.permission_type,
        })
    }
}

pub struct CheckTeacherPermissionMiddleware<S> {
    service: Rc<RefCell<S>>,
    permission_type: Option<PermissionType>,
}

impl<S, B> Service for CheckTeacherPermissionMiddleware<S>
where
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
            let teacher_id = TeacherID::from_request(&http_req, &mut payload).into_inner()?;

            let permission = TeacherPermission::of_entity(db.get_ref(), account_id, teacher_id)
                .await
                .map_err(|error| Error::from(APIError::from(error)))?;

            if expected_permission == Some(PermissionType::ReadWrite)
                && permission.permission_type == PermissionType::Read
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