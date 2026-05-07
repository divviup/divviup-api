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
    handler::{actor_required, ReplaceMimeTypes},
};
pub use health_check::health_check;
use trillium::{
    state, Handler,
    Method::{Delete, Get, Patch, Post},
};
use trillium_api::api;
use trillium_router::router;

pub fn routes(auth0_client: Auth0Client) -> impl Handler {
    router().any(
        &[Get, Post, Delete, Patch],
        "/api/*",
        (state(auth0_client), api_routes()),
    )
}

fn api_routes() -> impl Handler {
    // ReplaceMimeTypes stays in the outer chain because the trillium-router's
    // before_send does not replay path adjustments, so handlers inside route
    // entries never get their before_send called for wildcard matches. Keeping
    // it here means it also transforms headers for requests that fall through
    // to the Axum proxy, but that's harmless: Trillium already validated them,
    // and the Axum side accepts pre-transformed application/json.
    (
        ReplaceMimeTypes,
        api(actor_required),
        router().all("/admin/*", admin::routes()),
    )
}

pub(crate) mod axum_routes {
    use super::{
        accounts, aggregators::axum_handler as aggregators, api_tokens, collector_credentials,
        memberships, tasks::axum_handler as tasks, users,
    };
    use crate::handler::AxumAppState;
    use axum::routing::{delete, get};

    /// Axum sub-router for `/api` routes.
    ///
    /// During the proxy migration, Trillium's `ReplaceMimeTypes` handler in
    /// the outer chain already validates/normalizes request headers and sets
    /// the response Content-Type via `before_send`. So we do NOT apply
    /// `ReplaceMimeTypesLayer` here — it would reject the already-normalized
    /// `application/json` Content-Type that Trillium forwarded.
    ///
    /// TODO: wire `ReplaceMimeTypesLayer` once the Trillium proxy is removed.
    pub fn api_router() -> axum::Router<AxumAppState> {
        axum::Router::new()
            .route("/users/me", get(users::show))
            .route("/accounts", get(accounts::index).post(accounts::create))
            .route("/memberships/{membership_id}", delete(memberships::delete))
            .route(
                "/api_tokens/{api_token_id}",
                delete(api_tokens::delete).patch(api_tokens::update),
            )
            .route(
                "/collector_credentials/{collector_credential_id}",
                delete(collector_credentials::delete)
                    .get(collector_credentials::show)
                    .patch(collector_credentials::update),
            )
            .route(
                "/aggregators",
                get(aggregators::index_shared).post(aggregators::admin_create),
            )
            .route(
                "/aggregators/{aggregator_id}",
                get(aggregators::show)
                    .patch(aggregators::update)
                    .delete(aggregators::delete),
            )
            .route(
                "/tasks/{task_id}",
                get(tasks::show).patch(tasks::update).delete(tasks::delete),
            )
            .nest(
                "/accounts/{account_id}",
                axum::Router::new()
                    .route("/", get(accounts::show).patch(accounts::update))
                    .route(
                        "/memberships",
                        get(memberships::index).post(memberships::create),
                    )
                    .route(
                        "/api_tokens",
                        get(api_tokens::index).post(api_tokens::create),
                    )
                    .route(
                        "/collector_credentials",
                        get(collector_credentials::index).post(collector_credentials::create),
                    )
                    .route("/tasks", get(tasks::index).post(tasks::create))
                    .route(
                        "/aggregators",
                        get(aggregators::index_for_account).post(aggregators::create),
                    ),
            )
    }
}
