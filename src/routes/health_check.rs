use crate::Db;
use axum::{extract::State, http::StatusCode};
use sea_orm::ConnectionTrait;

pub async fn health_check(State(db): State<Db>) -> StatusCode {
    if db.execute_unprepared("select 1").await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}
