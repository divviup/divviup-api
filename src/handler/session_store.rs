use crate::{config::SessionSecrets, entity::session, Db};
use async_trait::async_trait;
use sea_orm::{
    sea_query::{any, OnConflict},
    ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use std::collections::HashMap;
use time::{Duration, OffsetDateTime};
use tower_sessions::{
    cookie::{Key, SameSite},
    service::SignedCookie,
    session::{Id as TowerSessionId, Record},
    session_store as tower_store, Expiry, SessionManagerLayer,
};

/// Cookie name used by the session middleware.
pub const SESSION_COOKIE_NAME: &str = "divviup.sid";

/// Database-backed session store for [`tower-sessions`].
///
/// Sessions are stored in the `sessions` table. Expired sessions are
/// cleaned up by the [`SessionCleanup`](crate::queue::SessionCleanup)
/// queue job rather than by the store itself.
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
/// `with_older_secrets` equivalent. Older secrets in the config are
/// parsed but ignored — see <https://github.com/divviup/divviup-api/issues/2252>.
pub fn axum_session_layer(
    db: Db,
    secrets: &SessionSecrets,
) -> SessionManagerLayer<TowerSessionStore, SignedCookie> {
    if !secrets.older.is_empty() {
        tracing::warn!(
            count = secrets.older.len(),
            "SESSION_SECRETS contains older keys that are ignored — \
             session cookie rotation is not yet supported, see \
             https://github.com/divviup/divviup-api/issues/2252"
        );
    }

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
