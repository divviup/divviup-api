use crate::{
    entity::{AccountColumn, Accounts, MembershipColumn, Memberships},
    handler::Error,
    Db,
};
use axum::extract::{FromRef, FromRequestParts, OptionalFromRequestParts};
use axum::http::request::Parts;
use sea_orm::{
    sea_query::all, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, Select,
};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use tower_sessions::Session;
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
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub admin: Option<bool>,
}

impl User {
    pub fn memberships(&self) -> Select<Memberships> {
        Memberships::for_user(self)
    }

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
            .is_some_and(|n| n > 0)
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
        conn.insert_state(user.clone());
        Some(user)
    }
}

// ---------------------------------------------------------------------------
// Axum extractor — mirrors the Trillium FromConn above
// ---------------------------------------------------------------------------

impl User {
    /// Inner helper for the extractor impls below.
    ///
    /// Returns `Ok(Some(user))` if a user is present in the session,
    /// `Ok(None)` if the session is empty, or `Err` for session-store errors.
    pub(crate) async fn from_parts<S>(parts: &mut Parts, state: &S) -> Result<Option<Self>, Error>
    where
        Db: FromRef<S>,
        S: Send + Sync,
    {
        // Cache: return early if already extracted and admin-populated.
        if let Some(user) = parts.extensions.get::<Self>() {
            if user.admin.is_some() {
                return Ok(Some(user.clone()));
            }
        }

        // Get a mutable user, preferring extensions and falling back to session.
        let mut user = if let Some(user) = parts.extensions.remove::<Self>() {
            user
        } else {
            let Some(session) = parts.extensions.get::<Session>() else {
                return Ok(None);
            };
            let Some(user) = session.get::<Self>(USER_SESSION_KEY).await? else {
                return Ok(None);
            };
            user
        };

        let db = Db::from_ref(state);
        user.populate_admin(&db).await;
        parts.extensions.insert(user.clone());
        Ok(Some(user))
    }
}

impl<S> FromRequestParts<S> for User
where
    Db: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Error> {
        Self::from_parts(parts, state)
            .await?
            .ok_or(Error::AccessDenied)
    }
}

impl<S> OptionalFromRequestParts<S> for User
where
    Db: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Option<Self>, Error> {
        Self::from_parts(parts, state).await
    }
}
