use crate::{
    entity::{AccountColumn, Accounts, MembershipColumn, Memberships},
    Db,
};
use sea_orm::{
    prelude::*,
    sea_query::{self, all},
    QuerySelect,
};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
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
    pub fn for_integration_testing() -> Self {
        Self {
            email: "integration@test.example".into(),
            email_verified: true,
            name: "integration testing user".into(),
            nickname: "test".into(),
            picture: None,
            sub: "".into(),
            updated_at: OffsetDateTime::now_utc() - Duration::days(1),
            admin: Some(true),
        }
    }

    async fn fetch_admin(&self, db: &Db) -> bool {
        Memberships::find()
            .inner_join(Accounts)
            .filter(all![
                MembershipColumn::UserEmail.eq(self.email.clone()),
                AccountColumn::Admin.eq(true)
            ])
            .limit(1)
            .count(db)
            .await
            .ok()
            .map_or(false, |n| n > 0)
    }

    async fn populate_admin(&mut self, db: &Db) {
        if self.admin.is_none() {
            self.admin = Some(self.fetch_admin(db).await);
        }
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
        let db: &Db = conn.state()?;
        user.populate_admin(db).await;
        conn.set_state(user.clone());
        Some(user)
    }
}
