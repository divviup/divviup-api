use crate::{entity::session, Db};
use async_session::{
    async_trait,
    chrono::{DateTime, NaiveDateTime, Utc},
    serde_json, Session,
};
use sea_orm::{
    sea_query::OnConflict, ColumnTrait, Condition, EntityTrait, IntoActiveModel, QueryFilter,
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

    fn try_from(session: &Session) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: session.id().to_string(),
            expiry: session
                .expiry()
                .map(|e| OffsetDateTime::from_unix_timestamp(e.timestamp()).unwrap()),
            data: serde_json::from_value(
                serde_json::to_value(session)?.get("data").unwrap().clone(),
            )?,
        })
    }
}

impl TryFrom<session::Model> for Session {
    type Error = serde_json::Error;
    fn try_from(db_session: session::Model) -> std::result::Result<Session, serde_json::Error> {
        let mut session: Session = serde_json::from_value(json!({
            "id": db_session.id,
            "data": db_session.data,
        }))?;
        if let Some(x) = db_session.expiry {
            session.set_expiry(DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(x.unix_timestamp(), x.nanosecond()).unwrap(),
                Utc,
            ))
        }
        Ok(session)
    }
}

#[async_trait]
impl async_session::SessionStore for SessionStore {
    async fn load_session(&self, cookie_value: String) -> async_session::Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;
        Ok(session::Entity::find_by_id(id)
            .filter(
                Condition::any()
                    .add(session::Column::Expiry.is_null())
                    .add(session::Column::Expiry.gt(OffsetDateTime::now_utc())),
            )
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
                    .to_owned(),
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
