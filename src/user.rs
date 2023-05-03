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
    #[cfg(feature = "kind-integration")]
    pub fn for_kind() -> Self {
        use std::time::Duration;

        Self {
            email: "kind@test.example".into(),
            email_verified: true,
            name: "kind user".into(),
            nickname: "kind".into(),
            picture: None,
            sub: "".into(),
            updated_at: OffsetDateTime::now_utc() - Duration::from_secs(24 * 60 * 60),
            admin: Some(false),
        }
    }

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
        let db = Db::from_conn(conn).await?;
        let mut user: Self = conn
            .take_state()
            .or_else(|| conn.session().get(USER_SESSION_KEY))?;
        user.populate_admin(&db).await;
        conn.set_state(user.clone());
        Some(user)
    }
}
