use crate::{
    entity::{AccountColumn, Accounts, MembershipColumn, Memberships},
    Db,
};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use trillium::{async_trait, Conn};
use trillium_api::FromConn;
use trillium_sessions::SessionConnExt;

pub const USER_SESSION_KEY: &str = "user";

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct User {
    pub email: String,
    pub email_verified: bool,
    pub name: String,
    pub nickname: String,
    pub picture: Option<String>,
    pub sub: String,
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
    pub admin: Option<bool>,
}

impl User {
    async fn populate_admin(&mut self, db: &Db) {
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
        println!("get user from conn");
        let db = Db::from_conn(conn).await?;
        println!("got DB");
        let mut user: Self = conn.take_state().or_else(|| {
            println!("get user from session");
            conn.session().get(USER_SESSION_KEY)
        })?;
        println!("got user {user:?}");
        user.populate_admin(&db).await;
        conn.set_state(user.clone());
        Some(user)
    }
}
