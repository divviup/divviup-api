/// Shared helper for Axum entity extractors.
///
/// Each "entity extractor" (Account, Task, Aggregator, ApiToken,
/// CollectorCredential) follows the same pattern:
///
///  1. Extract path parameter by name → parse as UUID
///  2. Extract [`PermissionsActor`] (bearer token **or** session user)
///  3. Look the entity up by primary key
///  4. Check permissions via [`Permissions`] trait
///
/// [`extract_entity`] captures this pattern as a single generic async
/// function, and the per-type [`FromRequestParts`] impls become one-liners.
use std::collections::HashMap;

use axum::extract::{FromRef, FromRequestParts, Path};
use axum::http::request::Parts;
use sea_orm::EntityTrait;
use uuid::Uuid;

use crate::{handler::Error, Db, Permissions, PermissionsActor};

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
/// * [`Error::NotFound`] — path param missing / unparseable, or no DB row
/// * [`Error::AccessDenied`] — actor lacks permission for the HTTP method
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
        .ok_or(Error::AccessDenied)
}
