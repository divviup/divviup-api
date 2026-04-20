use crate::{handler::Error, Config, PermissionsActor};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tower_sessions::Session;
use trillium::{Conn, Handler, Status};
use trillium_api::Halt;

pub async fn actor_required(_: &mut Conn, actor: Option<PermissionsActor>) -> impl Handler {
    if actor.is_none() {
        Some((Status::Forbidden, Halt))
    } else {
        None
    }
}

pub async fn admin_required(_: &mut Conn, actor: Option<PermissionsActor>) -> impl Handler {
    if matches!(actor, Some(actor) if actor.is_admin()) {
        None
    } else {
        // we return not found instead of forbidden so as to not
        // reveal what admin endpoints exist
        Some((Status::NotFound, Halt))
    }
}

/// `GET /logout` — destroy the session and redirect to Auth0's logout URL so
/// the IdP session is also cleared.
pub async fn logout(
    State(config): State<Arc<Config>>,
    session: Session,
) -> Result<Response, Error> {
    session.flush().await?;

    let mut logout_url = config.auth_url.join("/v2/logout")?;
    logout_url.query_pairs_mut().extend_pairs([
        ("client_id", &*config.auth_client_id),
        ("returnTo", config.app_url.as_ref()),
    ]);

    Ok((
        StatusCode::FOUND,
        [(header::LOCATION, logout_url.to_string())],
    )
        .into_response())
}
