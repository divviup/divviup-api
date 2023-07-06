mod accounts;
mod admin;
mod aggregators;
mod api_tokens;
mod health_check;
mod memberships;
mod tasks;
mod users;

use crate::{
    clients::Auth0Client,
    handler::{
        admin_required, destroy_session, logout_from_auth0,
        oauth2::{self, OauthClient},
        redirect_if_logged_in, user_required, ReplaceMimeTypes,
    },
    ApiConfig,
};
pub use health_check::health_check;
use trillium::{
    state, Handler,
    Method::{Delete, Get, Patch, Post},
};
use trillium_api::api;
use trillium_redirect::redirect;
use trillium_router::router;

pub fn routes(config: &ApiConfig) -> impl Handler {
    let oauth2_client = OauthClient::new(&config.oauth_config());
    let auth0_client = Auth0Client::new(config);

    router()
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
            (state(auth0_client), api_routes(config)),
        )
}

fn api_routes(config: &ApiConfig) -> impl Handler {
    (
        ReplaceMimeTypes,
        api(user_required),
        router()
            .get("/users/me", api(users::show))
            .get("/accounts", api(accounts::index))
            .post("/accounts", api(accounts::create))
            .delete("/memberships/:membership_id", api(memberships::delete))
            .get("/tasks/:task_id", api(tasks::show))
            .get(
                "/tasks/:task_id/collector_auth_tokens",
                api(tasks::collector_auth_tokens::index),
            )
            .patch("/tasks/:task_id", api(tasks::update))
            .patch("/aggregators/:aggregator_id", api(aggregators::update))
            .get("/aggregators/:aggregator_id", api(aggregators::show))
            .delete("/aggregators/:aggregator_id", api(aggregators::delete))
            .post(
                "/aggregators",
                (api(admin_required), api(aggregators::admin_create)),
            )
            .delete("/api_tokens/:api_token_id", api(api_tokens::delete))
            .patch("/api_tokens/:api_token_id", api(api_tokens::update))
            .any(
                &[Patch, Get, Post],
                "/accounts/:account_id/*",
                accounts_routes(config),
            )
            .all("/admin/*", admin::routes()),
    )
}

fn accounts_routes(config: &ApiConfig) -> impl Handler {
    router()
        .patch("/", api(accounts::update))
        .get("/", api(accounts::show))
        .get("/memberships", api(memberships::index))
        .post("/memberships", api(memberships::create))
        .get("/tasks", api(tasks::index))
        .post("/tasks", (state(config.clone()), api(tasks::create)))
        .post("/aggregators", api(aggregators::create))
        .get("/aggregators", api(aggregators::index))
        .post("/api_tokens", api(api_tokens::create))
        .get("/api_tokens", api(api_tokens::index))
}
