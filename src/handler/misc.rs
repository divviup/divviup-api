use crate::PermissionsActor;
use trillium::{Conn, Handler, Status};
use trillium_api::Halt;

pub async fn actor_required(_: &mut Conn, actor: Option<PermissionsActor>) -> impl Handler {
    if actor.is_none() {
        Some((Status::Forbidden, Halt))
    } else {
        None
    }
}

pub async fn admin_required(_: &mut Conn, actor: Option<PermissionsActor>) -> impl Handler {
    if matches!(actor, Some(actor) if actor.is_admin()) {
        None
    } else {
        // we return not found instead of forbidden so as to not
        // reveal what admin endpoints exist
        Some((Status::NotFound, Halt))
    }
}
