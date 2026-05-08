use crate::{
    entity::queue::{self, Column, Entity, JobStatus, Model},
    handler::extract::Json,
    Db, Error, PermissionsActor,
};
use axum::extract::{FromRef, FromRequestParts, Path, Query, Request, State};
use axum::http::{header, request::Parts, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use httpdate::fmt_http_date;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryOrder, QuerySelect};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

impl<S> FromRequestParts<S> for queue::Model
where
    Db: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Error> {
        let Path(params) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::NotFound)?;

        let id = params
            .get("job_id")
            .and_then(|s| s.parse::<Uuid>().ok())
            .ok_or(Error::NotFound)?;

        let db = Db::from_ref(state);
        Entity::find_by_id(id)
            .one(&db)
            .await?
            .ok_or(Error::NotFound)
    }
}

#[derive(Deserialize)]
pub struct IndexParams {
    status: Option<JobStatus>,
}

pub mod axum_handler {
    use super::*;

    pub async fn require_admin(
        actor: PermissionsActor,
        request: Request,
        next: Next,
    ) -> axum::response::Response {
        if actor.is_admin() {
            next.run(request).await
        } else {
            StatusCode::NOT_FOUND.into_response()
        }
    }

    pub async fn index(
        State(db): State<Db>,
        Query(params): Query<IndexParams>,
    ) -> Result<Json<Vec<Model>>, Error> {
        let mut find = Entity::find();
        if let Some(status) = params.status {
            let query = QuerySelect::query(&mut find);
            query.cond_where(Column::Status.eq(status));
        }

        Ok(Json(
            find.order_by_desc(Column::UpdatedAt)
                .limit(100)
                .all(&db)
                .await?,
        ))
    }

    pub async fn show(queue_job: Model) -> impl IntoResponse {
        let last_modified = fmt_http_date(queue_job.updated_at.into());
        ([(header::LAST_MODIFIED, last_modified)], Json(queue_job))
    }

    pub async fn delete(queue_job: Model, State(db): State<Db>) -> Result<StatusCode, Error> {
        queue_job.delete(&db).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
