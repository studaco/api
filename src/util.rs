use serde::{Deserializer, Deserialize};

pub fn deserialize_optional_field<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

#[macro_export]
macro_rules! uuid_wrapper {
    ($type:ident) => {
        #[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone, Eq, PartialEq, sqlx::Type)]
        #[sqlx(transparent)]
        pub struct $type(uuid::Uuid);

        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl actix_web::FromRequest for $type {
            type Error = crate::error::APIError;
            type Future = futures::future::Ready<Result<Self, Self::Error>>;
            type Config = ();

            fn from_request(req: &actix_web::HttpRequest, _: &mut actix_http::Payload) -> Self::Future {
                futures::future::ready(
                    req.extensions()
                        .get::<Self>()
                        .map(Self::clone)
                        .ok_or(crate::error::APIError::InternalError {
                            message: "Error encountered while extracting parameters".to_string(),
                        }),
                )
            }
        }

    }
}
