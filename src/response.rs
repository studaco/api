use crate::error::APIError;
use crate::payload::Payload;
use serde::Serialize;

#[derive(Serialize)]
pub struct APIResponse<Response>
where
    Response: Serialize,
{
    ok: bool,
    payload: Option<Payload<Response>>,
    error: Option<APIError>,
}

impl<T: Serialize> From<APIError> for APIResponse<T> {
    fn from(error: APIError) -> APIResponse<T> {
        APIResponse {
            ok: false,
            payload: None,
            error: Some(error),
        }
    }
}

impl<T: Serialize> From<Payload<T>> for APIResponse<T> {
    fn from(payload: Payload<T>) -> APIResponse<T> {
        APIResponse {
            ok: true,
            payload: Some(payload),
            error: None,
        }
    }
}

impl<T: Serialize> From<Result<Payload<T>, APIError>> for APIResponse<T> {
    fn from(result: Result<Payload<T>, APIError>) -> APIResponse<T> {
        match result {
            Ok(payload) => payload.into(),
            Err(error) => error.into(),
        }
    }
}

impl<T: Serialize> APIResponse<T> {
    fn payload(of: T) -> APIResponse<T> {
        Payload::from(of).into()
    }
}

impl<T: Serialize> std::ops::Try for APIResponse<T> {
    type Ok = T;
    type Error = APIError;

    fn into_result(self) -> std::result::Result<T, APIError> {
        match self {
            APIResponse {
                ok: false,
                payload: None,
                error: Some(error),
            } => Err(error),
            APIResponse {
                ok: true,
                payload: Some(Payload { payload }),
                error: None,
            } => Ok(payload),
            _ => panic!("Invalid construction of API response"),
        }
    }

    fn from_error(v: APIError) -> Self {
        v.into()
    }

    fn from_ok(v: T) -> Self {
        Payload::from(v).into()
    }
}