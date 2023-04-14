mod accounts;
mod health_check;
mod memberships;
mod tasks;
mod users;

use crate::{
    clients::Auth0Client,
    handler::{
        destroy_session, logout_from_auth0,
        oauth2::{self, OauthClient},
        redirect_if_logged_in, user_required, ReplaceMimeTypes,
    },
    ApiConfig,
};
use health_check::health_check;
use trillium::{
    state, Handler,
    Method::{Delete, Get, Patch, Post},
};
use trillium_api::api;
use trillium_redirect::redirect;
use trillium_router::router;

pub fn routes(config: &ApiConfig) -> impl Handler {
    let oauth2_client = OauthClient::new(&config.oauth_config());
    let auth0_client =
        Auth0Client::new(config).with_http_client(oauth2_client.http_client().clone());

    router()
        .get("/health", api(health_check))
        .get(
            "/login",
            (
                redirect_if_logged_in(config),
                state(oauth2_client.clone()),
                oauth2::redirect,
            ),
        )
        .get("/logout", (destroy_session, logout_from_auth0(config)))
        .get(
            "/callback",
            (
                state(oauth2_client),
                oauth2::callback,
                redirect(config.app_url.to_string()),
            ),
        )
        .any(
            &[Get, Post, Delete, Patch],
            "/api/*",
            (state(auth0_client), api_routes()),
        )
}

fn api_routes() -> impl Handler {
    (
        ReplaceMimeTypes,
        api(user_required),
        router()
            .get("/users/me", api(users::show))
            .get("/accounts", api(accounts::index))
            .post("/accounts", api(accounts::create))
            .delete("/memberships/:membership_id", api(memberships::delete))
            .get("/tasks/:task_id", api(tasks::show))
            .patch("/tasks/:task_id", api(tasks::update))
            .any(
                &[Patch, Get, Post],
                "/accounts/:account_id/*",
                accounts_routes(),
            ),
    )
}

fn accounts_routes() -> impl Handler {
    router()
        .patch("/", api(accounts::update))
        .get("/", api(accounts::show))
        .get("/memberships", api(memberships::index))
        .post("/memberships", api(memberships::create))
        .get("/tasks", api(tasks::index))
        .post("/tasks", api(tasks::create))
}
