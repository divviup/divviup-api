/// Shared helpers for Axum extractors and responses.
use std::collections::HashMap;

use axum::extract::{FromRef, FromRequest, FromRequestParts, Path};
use axum::http::{request::Parts, StatusCode};
use axum::response::{IntoResponse, Response};
use sea_orm::EntityTrait;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use crate::{handler::Error, Db, Permissions, PermissionsActor};

/// A JSON extractor/response that mirrors the Trillium `api()` + `Json<T>`
/// behaviour: request bodies are deserialized via [`serde_path_to_error`] so
/// that parse failures produce the same `{"path":…,"message":…}` error shape.
pub struct Json<T>(pub T);

impl<T, S> FromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = axum::body::Bytes::from_request(req, state).await.map_err(|e| {
            if e.status() == StatusCode::PAYLOAD_TOO_LARGE {
                Error::PayloadTooLarge
            } else {
                Error::Other(std::sync::Arc::new(e))
            }
        })?;
        let deserializer = &mut serde_json::Deserializer::from_slice(&bytes);
        serde_path_to_error::deserialize(deserializer)
            .map(Json)
            .map_err(|err| {
                Error::Json(trillium_api::Error::ParseError {
                    path: err.path().to_string(),
                    message: err.inner().to_string(),
                })
            })
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

/// Look up an entity by a named path parameter, check permissions, and
/// return it — or an appropriate [`Error`].
///
/// `E` is the Sea-ORM **Entity** type (e.g. `Accounts`). Its `Model` must
/// implement [`Permissions`] and [`Clone`].
///
/// **Note**: This always requires a valid [`PermissionsActor`], so it cannot
/// be used for truly public (unauthenticated) entity endpoints. If such a
/// route is needed in the future, extract the entity and check permissions
/// separately.
///
/// # Errors
///
/// * [`Error::NotFound`] — path param missing / unparseable, no DB row,
///   or the actor lacks permission (intentionally hides resource existence)
/// * Propagates DB errors and [`PermissionsActor`] extraction failures
pub async fn extract_entity<E, S>(
    parts: &mut Parts,
    state: &S,
    param_name: &str,
) -> Result<E::Model, Error>
where
    E: EntityTrait,
    <E::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType: From<Uuid>,
    E::Model: Permissions + Clone,
    Db: FromRef<S>,
    S: Send + Sync,
{
    let Path(params) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
        .await
        .map_err(|_| Error::NotFound)?;

    let id = params
        .get(param_name)
        .and_then(|s| s.parse::<Uuid>().ok())
        .ok_or(Error::NotFound)?;

    let actor = PermissionsActor::from_request_parts(parts, state).await?;
    let db = Db::from_ref(state);

    let entity = E::find_by_id(id).one(&db).await?.ok_or(Error::NotFound)?;

    actor
        .if_allowed_http(&parts.method, entity)
        .ok_or(Error::NotFound)
}
