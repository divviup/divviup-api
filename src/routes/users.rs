use crate::{handler::extract::Json, User};

pub async fn show(user: User) -> Json<User> {
    Json(user)
}
