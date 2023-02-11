use crate::{handler::oauth2::OauthClient, ApiConfig, User};

use trillium::{Conn, Handler, Status};
use trillium_api::{api, Halt};

use trillium_redirect::Redirect;
use trillium_sessions::SessionConnExt;

/// note(jbr): most of these need to find better places to live
pub fn redirect_if_logged_in(config: &ApiConfig) -> impl Handler {
    let app_url = config.app_url.clone().to_string();
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
        ("client_id", config.auth_client_id.clone()),
        ("returnTo", config.app_url.to_string()),
    ]);

    Redirect::to(logout_url.to_string())
}

pub async fn destroy_session(mut conn: Conn) -> Conn {
    conn.session_mut().destroy();
    conn
}

pub fn populate_oauth2_client(config: &ApiConfig) -> impl Handler {
    trillium::state(OauthClient::new(&config.oauth_config()))
}

pub async fn user_required(_: &mut Conn, user: Option<User>) -> impl Handler {
    if user.is_none() {
        Some((Status::Forbidden, Halt))
    } else {
        None
    }
}
