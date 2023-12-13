use crate::Error;
use std::borrow::Cow;
use trillium_router::RouterConnExt;

fn error_type(value: &Error) -> Option<Cow<'static, str>> {
    match value {
        Error::Database(_) => Some("DbError"),
        Error::Client(_) => Some("ClientError"),
        Error::Other(_) => Some("Other"),
        Error::NumericConversion(_) => None,
        Error::TimeComponentRange(_) => Some("TimeComponentRangeError"),
        Error::TaskProvisioning(_) => Some("TaskProvisioningError"),
        Error::Encryption => Some("EncryptionError"),
        Error::String(s) => Some(*s),
        Error::Codec(_) => Some("CodecError"),
        _ => None,
    }
    .map(Cow::Borrowed)
}

#[cfg(feature = "otlp-trace")]
pub fn opentelemetry() -> impl trillium::Handler {
    trillium_opentelemetry::global::instrument()
        .with_route(|conn| conn.route().map(|r| r.to_string().into()))
        .with_error_type(|conn| conn.state().and_then(error_type))
        .with_headers([trillium::KnownHeaderName::XrequestId])
}

#[cfg(not(feature = "otlp-trace"))]
pub fn opentelemetry() -> impl trillium::Handler {
    trillium_opentelemetry::global::metrics()
        .with_route(|conn| conn.route().map(|r| r.to_string().into()))
        .with_error_type(|conn| conn.state().and_then(error_type))
}
