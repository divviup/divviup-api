use crate::{ApiConfig, PermissionsActor, User};
use trillium::{Conn, Handler, Status};
use trillium_api::{api, Halt};
use trillium_redirect::Redirect;
use trillium_sessions::SessionConnExt;

/// note(jbr): most of these need to find better places to live
pub fn redirect_if_logged_in(config: &ApiConfig) -> impl Handler {
    let app_url = config.app_url.to_string();
    api(move |_: &mut Conn, user: Option<User>| {
        let app_url = app_url.clone();
        async move {
            if user.is_some() {
                Some(Redirect::to(app_url.clone()))
            } else {
                None
            }
        }
    })
}

pub fn logout_from_auth0(config: &ApiConfig) -> impl Handler {
    let mut logout_url = config.auth_url.join("/v2/logout").unwrap();

    logout_url.query_pairs_mut().extend_pairs([
        ("client_id", &*config.auth_client_id),
        ("returnTo", config.app_url.as_ref()),
    ]);

    Redirect::to(logout_url.to_string())
}

pub async fn destroy_session(mut conn: Conn) -> Conn {
    conn.session_mut().destroy();
    conn
}

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
