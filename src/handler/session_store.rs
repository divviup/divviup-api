use crate::{config::SessionSecrets, entity::session, Db};
use async_session::{
    async_trait,
    chrono::{DateTime, Utc},
    Session,
};
use sea_orm::{
    sea_query::{any, OnConflict},
    ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use serde_json::json;
use std::collections::HashMap;
use time::{Duration, OffsetDateTime};
use tower_sessions::{
    cookie::{Key, SameSite},
    service::SignedCookie,
    session::{Id as TowerSessionId, Record},
    session_store as tower_store, Expiry, SessionManagerLayer,
};

/// Cookie name shared with the Trillium-side session middleware. Both sides
/// set a cookie with this name; the wire formats are incompatible, so each
/// side only successfully parses its own.
// TODO: remove this comment when Trillium is removed — the incompatibility
// note will no longer apply.
pub const SESSION_COOKIE_NAME: &str = "divviup.sid";

#[derive(Debug, Clone)]
pub struct SessionStore {
    db: Db,
}

impl SessionStore {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

impl TryFrom<&Session> for session::Model {
    type Error = serde_json::Error;

    fn try_from(session: &Session) -> Result<Self, Self::Error> {
        Ok(Self {
            id: session.id().to_string(),
            // unwrap safety: session object comes from the session handler, and its timestamp
            // we made ourselves.
            expiry: session
                .expiry()
                .map(|e| OffsetDateTime::from_unix_timestamp(e.timestamp()).unwrap()),
            // unwrap safety: if the serialization is successful, the data element
            // will be there.
            data: serde_json::from_value(
                serde_json::to_value(session)?.get("data").unwrap().clone(),
            )?,
        })
    }
}

impl TryFrom<session::Model> for Session {
    type Error = serde_json::Error;
    fn try_from(db_session: session::Model) -> Result<Session, serde_json::Error> {
        let mut session: Session = serde_json::from_value(json!({
            "id": db_session.id,
            "data": db_session.data,
        }))?;
        if let Some(x) = db_session.expiry {
            // unwrap safety: the expiry time from the database is a timestamp we made
            // ourselves.
            session.set_expiry(
                DateTime::<Utc>::from_timestamp(x.unix_timestamp(), x.nanosecond()).unwrap(),
            );
        }
        Ok(session)
    }
}

#[async_trait]
impl async_session::SessionStore for SessionStore {
    async fn load_session(&self, cookie_value: String) -> async_session::Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;
        Ok(session::Entity::find_by_id(id)
            .filter(any![
                session::Column::Expiry.is_null(),
                session::Column::Expiry.gt(OffsetDateTime::now_utc())
            ])
            .one(&self.db)
            .await?
            .map(Session::try_from)
            .transpose()?)
    }

    async fn store_session(&self, session: Session) -> async_session::Result<Option<String>> {
        let session_model = session::Model::try_from(&session)?.into_active_model();

        session::Entity::insert(session_model)
            .on_conflict(
                OnConflict::column(session::Column::Id)
                    .update_columns([session::Column::Data, session::Column::Expiry])
                    .clone(),
            )
            .exec(&self.db)
            .await?;

        Ok(session.into_cookie_value())
    }

    async fn destroy_session(&self, session: Session) -> async_session::Result {
        session::Entity::delete_by_id(session.id())
            .exec(&self.db)
            .await?;
        Ok(())
    }

    async fn clear_store(&self) -> async_session::Result {
        session::Entity::delete_many().exec(&self.db).await?;
        Ok(())
    }
}

/// Axum-side session store implementing [`tower_sessions_core::SessionStore`].
///
/// Uses the same `session` database table as the Trillium-side [`SessionStore`],
/// but with a different session ID format (`tower-sessions` uses base64-encoded
/// `i128` rather than `async_session`'s UUID strings).
// TODO: remove the Trillium-side `SessionStore` and the `async_session`
// dependency when Trillium is removed.
#[derive(Debug, Clone)]
pub struct TowerSessionStore {
    db: Db,
}

impl TowerSessionStore {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

/// Build the Axum-side [`SessionManagerLayer`].
///
/// `tower-sessions` 0.15 accepts only a single signing key; there is no
/// `with_older_secrets` equivalent. Key rotation for the Axum cookie will be
/// revisited once the Trillium server is removed in Part 8.
pub fn axum_session_layer(
    db: Db,
    secrets: &SessionSecrets,
) -> SessionManagerLayer<TowerSessionStore, SignedCookie> {
    // `cookie::Key::from` panics on keys shorter than 64 bytes. We only
    // guarantee 32, so derive a longer key from the configured secret.
    let key = Key::derive_from(&secrets.current);
    SessionManagerLayer::new(TowerSessionStore::new(db))
        .with_name(SESSION_COOKIE_NAME)
        .with_secure(true)
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_path("/")
        .with_signed(key)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)))
}

#[async_trait]
impl tower_store::SessionStore for TowerSessionStore {
    async fn save(&self, record: &Record) -> tower_store::Result<()> {
        let model = session::Model {
            id: record.id.to_string(),
            expiry: Some(record.expiry_date),
            data: serde_json::to_value(&record.data)
                .map_err(|e| tower_store::Error::Encode(e.to_string()))?,
        };

        session::Entity::insert(model.into_active_model())
            .on_conflict(
                OnConflict::column(session::Column::Id)
                    .update_columns([session::Column::Data, session::Column::Expiry])
                    .clone(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| tower_store::Error::Backend(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, session_id: &TowerSessionId) -> tower_store::Result<Option<Record>> {
        let model = session::Entity::find_by_id(session_id.to_string())
            .filter(any![
                session::Column::Expiry.is_null(),
                session::Column::Expiry.gt(OffsetDateTime::now_utc())
            ])
            .one(&self.db)
            .await
            .map_err(|e| tower_store::Error::Backend(e.to_string()))?;

        model
            .map(|m| {
                let data: HashMap<String, serde_json::Value> = serde_json::from_value(m.data)
                    .map_err(|e| tower_store::Error::Decode(e.to_string()))?;
                let id: TowerSessionId = m.id.parse().map_err(|e: base64::DecodeSliceError| {
                    tower_store::Error::Decode(e.to_string())
                })?;
                Ok(Record {
                    id,
                    data,
                    expiry_date: m.expiry.unwrap_or_else(|| {
                        // The DB allows null expiry, but tower-sessions requires a
                        // value. Use a far-future sentinel.
                        OffsetDateTime::now_utc() + time::Duration::weeks(52)
                    }),
                })
            })
            .transpose()
    }

    async fn delete(&self, session_id: &TowerSessionId) -> tower_store::Result<()> {
        session::Entity::delete_by_id(session_id.to_string())
            .exec(&self.db)
            .await
            .map_err(|e| tower_store::Error::Backend(e.to_string()))?;
        Ok(())
    }
}
