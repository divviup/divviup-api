mod accounts;
mod admin;
mod aggregators;
mod api_tokens;
mod collector_credentials;
mod health_check;
mod memberships;
mod tasks;
mod users;

use crate::{
    clients::Auth0Client,
    handler::{
        actor_required, admin_required, destroy_session, logout_from_auth0,
        oauth2::{self, OauthClient},
        redirect_if_logged_in, ReplaceMimeTypes,
    },
    Config,
};
pub use health_check::health_check;
use trillium::{
    state, Handler,
    Method::{Delete, Get, Patch, Post},
};
use trillium_api::api;
use trillium_redirect::redirect;
use trillium_router::router;

pub fn routes(config: &Config) -> impl Handler {
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
        .get("/tasks/:task_id", api(tasks::public_show))
        .any(
            &[Get, Post, Delete, Patch],
            "/api/*",
            (state(auth0_client), api_routes()),
        )
}

fn api_routes() -> impl Handler {
    (
        ReplaceMimeTypes,
        api(actor_required),
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
            .get("/aggregators", api(aggregators::index))
            .post(
                "/aggregators",
                (api(admin_required), api(aggregators::admin_create)),
            )
            .delete("/api_tokens/:api_token_id", api(api_tokens::delete))
            .patch("/api_tokens/:api_token_id", api(api_tokens::update))
            .delete(
                "/collector_credentials/:collector_credential_id",
                api(collector_credentials::delete),
            )
            .get(
                "/collector_credentials/:collector_credential_id",
                api(collector_credentials::show),
            )
            .patch(
                "/collector_credentials/:collector_credential_id",
                api(collector_credentials::update),
            )
            .any(
                &[Patch, Get, Post],
                "/accounts/:account_id/*",
                accounts_routes(),
            )
            .all("/admin/*", admin::routes()),
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
        .post("/aggregators", api(aggregators::create))
        .get("/aggregators", api(aggregators::index))
        .post("/api_tokens", api(api_tokens::create))
        .get("/api_tokens", api(api_tokens::index))
        .get("/collector_credentials", api(collector_credentials::index))
        .post("/collector_credentials", api(collector_credentials::create))
}
