use crate::Db;
use sea_orm::ConnectionTrait;
use trillium::{async_trait, Conn, Handler, Status};

struct HealthCheck(Db);

#[async_trait]
impl Handler for HealthCheck {
    async fn run(&self, conn: Conn) -> Conn {
        if conn.path() != "/health" {
            return conn;
        }
        if self.0.execute_unprepared("select 1").await.is_err() {
            return conn.halt().with_status(500);
        }
        conn.halt().with_status(Status::Ok)
    }
}

pub fn health_check(db: &Db) -> impl Handler {
    HealthCheck(db.clone())
}
