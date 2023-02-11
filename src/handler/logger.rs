use crate::user::User;
use std::borrow::Cow;
use trillium::{Conn, Handler, KnownHeaderName};
use trillium_logger::dev_formatter;

fn redirect_url(conn: &Conn, _color: bool) -> Cow<'static, str> {
    match conn
        .inner()
        .response_headers()
        .get(KnownHeaderName::Location)
    {
        Some(h) => format!(" -> {h}").into(),
        None => "".into(),
    }
}

fn user(conn: &Conn, _color: bool) -> String {
    match conn.state::<User>() {
        Some(User {
            email,
            admin: Some(true),
            ..
        }) => format!("{email} (admin)"),
        Some(User { email, .. }) => String::from(email),
        None => String::from("-"),
    }
}
pub fn logger() -> impl Handler {
    trillium_logger::logger().with_formatter((user, ": ", dev_formatter, redirect_url))
}
