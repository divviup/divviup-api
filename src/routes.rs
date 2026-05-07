mod accounts;
mod admin;
mod aggregators;
mod api_tokens;
mod collector_credentials;
mod health_check;
mod memberships;
mod tasks;
mod users;

pub use health_check::health_check;

pub(crate) mod axum_routes {
    use super::{
        accounts, admin::axum_handler as admin, aggregators::axum_handler as aggregators,
        api_tokens, collector_credentials, memberships, tasks::axum_handler as tasks, users,
    };
    use crate::handler::{custom_mime_types::ReplaceMimeTypesLayer, AxumAppState};
    use axum::routing::{delete, get};

    /// Axum sub-router for `/api` routes.
    pub fn api_router(state: &AxumAppState) -> axum::Router<AxumAppState> {
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
                "/admin",
                axum::Router::new()
                    .route("/queue", get(admin::index))
                    .route("/queue/{job_id}", get(admin::show).delete(admin::delete))
                    .route_layer(axum::middleware::from_fn_with_state(
                        state.clone(),
                        admin::require_admin,
                    )),
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
            .layer(ReplaceMimeTypesLayer)
    }
}
