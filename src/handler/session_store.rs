use crate::{entity::session, Db};
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
use time::OffsetDateTime;

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
