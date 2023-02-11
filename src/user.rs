use crate::{
    entity::{AccountColumn, Accounts, MembershipColumn, Memberships},
    DbConnExt,
};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use trillium::{async_trait, Conn};
use trillium_api::FromConn;
use trillium_sessions::SessionConnExt;

pub const USER_SESSION_KEY: &str = "user";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub email: String,
    email_verified: bool,
    name: String,
    nickname: String,
    picture: String,
    sub: String,
    #[serde(with = "time::serde::iso8601")]
    updated_at: OffsetDateTime,
    pub(crate) admin: Option<bool>,
}

impl User {
    async fn populate_admin(&mut self, db: &DbConn) {
        let membership = Memberships::find()
            .inner_join(Accounts)
            .filter(MembershipColumn::UserEmail.eq(String::from(&self.email)))
            .filter(AccountColumn::Admin.eq(true))
            .one(db)
            .await
            .ok()
            .flatten();

        self.admin = Some(membership.is_some());
    }

    pub fn is_admin(&self) -> bool {
        self.admin == Some(true)
    }
}

#[async_trait]
impl FromConn for User {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        let mut user: Self = conn
            .take_state()
            .or_else(|| conn.session().get(USER_SESSION_KEY))?;
        user.populate_admin(conn.db()).await;
        conn.set_state(user.clone());
        Some(user)
    }
}
