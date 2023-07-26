use super::random_chars;
use crate::{clients::auth0_client::Token, User};
use serde_json::json;
use trillium::Handler;
use trillium_api::{Halt, Json};
use trillium_router::router;

pub fn mock(auth0_url: &str) -> impl Handler {
    (
        router()
            .get("/userinfo", Json(User::for_integration_testing()))
            .post(
                "/oauth/token",
                Json(Token {
                    access_token: "access token".into(),
                    expires_in: 60,
                    scope: "".into(),
                    token_type: "bearer".into(),
                }),
            )
            .post(
                "/api/v2/users",
                Json(json!({ "user_id": random_chars(10) })),
            )
            .post(
                "/api/v2/tickets/password-change",
                Json(json!({
                    "ticket": format!("{auth0_url}/password_tickets/{}", random_chars(10))
                })),
            ),
        Halt,
    )
}
