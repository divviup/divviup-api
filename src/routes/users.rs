use crate::user::User;
use trillium::Conn;
use trillium_api::Json;

pub async fn show(_: &mut Conn, user: User) -> Json<User> {
    Json(user)
}
