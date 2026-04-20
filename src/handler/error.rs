use crate::clients::ClientError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json as AxumJson;
use sea_orm::DbErr;
use serde_json::json;
use std::{backtrace::Backtrace, sync::Arc};
use trillium::{async_trait, Conn, Handler, Status};
use trillium_api::{ApiConnExt, Error as ApiError};
use validator::ValidationErrors;

pub struct ErrorHandler;
#[async_trait]
impl Handler for ErrorHandler {
    async fn run(&self, conn: Conn) -> Conn {
        conn
    }

    async fn before_send(&self, mut conn: Conn) -> Conn {
        if let Some(error) = conn.take_state::<ApiError>() {
            conn.insert_state(Error::from(error));
        };

        let Some(error) = conn.state().cloned() else {
            return conn;
        };

        match error {
            Error::AccessDenied => conn.with_status(Status::Forbidden).with_body(""),

            Error::NotFound => conn.with_status(Status::NotFound).with_body(""),

            Error::Json(e @ ApiError::UnsupportedMimeType { .. }) => conn
                .with_status(Status::NotAcceptable)
                .with_body(e.to_string()),

            Error::Json(ApiError::ParseError { path, message }) => conn
                .with_status(Status::BadRequest)
                .with_json(&json!({"path": path, "message": message})),

            Error::Validation(e) => conn.with_status(Status::BadRequest).with_json(&e),

            e => {
                let string = e.to_string();
                log::error!("{e}");
                let mut conn = conn.with_status(Status::InternalServerError);
                if cfg!(debug_assertions) {
                    conn.with_body(string)
                } else {
                    conn.inner_mut().take_response_body();
                    conn
                }
            }
        }
    }
}

#[derive(thiserror::Error, Debug, Clone)]
#[non_exhaustive]
pub enum Error {
    #[error("Access denied")]
    AccessDenied,
    #[error(transparent)]
    Database(#[from] Arc<DbErr>),
    #[error("Not found")]
    NotFound,
    #[error(transparent)]
    Json(#[from] ApiError),
    #[error(transparent)]
    Validation(#[from] ValidationErrors),
    #[error(transparent)]
    Client(#[from] Arc<ClientError>),
    #[error(transparent)]
    Other(#[from] Arc<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    #[error(transparent)]
    NumericConversion(#[from] std::num::TryFromIntError),
    #[error(transparent)]
    TimeComponentRange(#[from] time::error::ComponentRange),
    #[error(transparent)]
    TaskProvisioning(#[from] crate::entity::task::TaskProvisioningError),
    #[error(transparent)]
    Uuid(#[from] uuid::Error),
    #[error("encryption error")]
    Encryption,
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    String(&'static str),
    #[error(transparent)]
    Codec(Arc<janus_messages::codec::CodecError>),
    #[error("csrf mismatch or missing")]
    CallbackCsrfMismatch,
    #[error("expected pkce verifier in session")]
    CallbackMissingPkce,
    #[error("expected code query param")]
    CallbackMissingCode,
}

impl From<janus_messages::codec::CodecError> for Error {
    fn from(error: janus_messages::codec::CodecError) -> Self {
        Self::Codec(Arc::new(error))
    }
}

impl From<aes_gcm::Error> for Error {
    fn from(_: aes_gcm::Error) -> Self {
        Self::Encryption
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(value: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Other(Arc::from(value))
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        ApiError::from(value).into()
    }
}

impl From<tower_sessions::session::Error> for Error {
    fn from(value: tower_sessions::session::Error) -> Self {
        Self::Other(Arc::new(value))
    }
}

impl From<DbErr> for Error {
    fn from(value: DbErr) -> Self {
        Self::Database(Arc::new(value))
    }
}

impl From<ClientError> for Error {
    fn from(value: ClientError) -> Self {
        Self::Client(Arc::new(value))
    }
}

#[async_trait]
impl Handler for Error {
    async fn run(&self, conn: Conn) -> Conn {
        conn.with_state(self.clone())
            .with_state(Backtrace::capture())
    }
}

/// Axum-side error-to-response conversion, mirroring the Trillium
/// [`ErrorHandler::before_send`] logic above.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::AccessDenied => StatusCode::FORBIDDEN.into_response(),

            Error::CallbackCsrfMismatch
            | Error::CallbackMissingPkce
            | Error::CallbackMissingCode => {
                // Preserve the Trillium-side behavior of 403 with a plain-text
                // explanatory body for OAuth callback validation failures.
                (StatusCode::FORBIDDEN, self.to_string()).into_response()
            }

            Error::NotFound => StatusCode::NOT_FOUND.into_response(),

            Error::Json(ApiError::UnsupportedMimeType { .. }) => {
                StatusCode::NOT_ACCEPTABLE.into_response()
            }

            Error::Json(ApiError::ParseError { path, message }) => (
                StatusCode::BAD_REQUEST,
                AxumJson(json!({"path": path, "message": message})),
            )
                .into_response(),

            Error::Validation(e) => (StatusCode::BAD_REQUEST, AxumJson(e)).into_response(),

            e => {
                log::error!("{e}");
                if cfg!(debug_assertions) {
                    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[test]
    fn access_denied_is_403() {
        let resp = Error::AccessDenied.into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn not_found_is_404() {
        let resp = Error::NotFound.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn unsupported_mime_type_is_406() {
        let err = Error::Json(ApiError::UnsupportedMimeType {
            mime_type: "text/plain".into(),
        });
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE);
    }

    #[test]
    fn parse_error_is_400() {
        let err = Error::Json(ApiError::ParseError {
            path: ".field".into(),
            message: "expected string".into(),
        });
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn validation_error_is_400() {
        let err = Error::Validation(ValidationErrors::new());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn other_error_is_500() {
        let err = Error::String("something broke");
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn callback_validation_errors_are_403() {
        for err in [
            Error::CallbackCsrfMismatch,
            Error::CallbackMissingPkce,
            Error::CallbackMissingCode,
        ] {
            let resp = err.into_response();
            assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        }
    }
}
