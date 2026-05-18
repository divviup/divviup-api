use super::random_chars;
use crate::{clients::auth0_client::Token, User};
use axum::{routing, Json, Router};
use serde_json::json;

pub fn mock(auth0_url: &str) -> Router {
    let auth0_url = auth0_url.to_owned();
    Router::new()
        .route(
            "/userinfo",
            routing::get(|| async { Json(User::for_integration_testing()) }),
        )
        .route(
            "/oauth/token",
            routing::post(|| async {
                Json(Token {
                    access_token: "access token".into(),
                    expires_in: 60,
                    scope: "".into(),
                    token_type: "bearer".into(),
                })
            }),
        )
        .route(
            "/api/v2/users",
            routing::post(|| async { Json(json!({ "user_id": random_chars(10) })) }),
        )
        .route(
            "/api/v2/tickets/password-change",
            routing::post(move || async move {
                Json(json!({
                    "ticket": format!("{auth0_url}/password_tickets/{}", random_chars(10))
                }))
            }),
        )
}
