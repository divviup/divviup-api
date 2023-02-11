use crate::DbConnExt;
use sqlx::Executor;
use trillium::Conn;

pub async fn health_check(conn: Conn) -> Conn {
    let db_healthy = conn
        .db()
        .get_postgres_connection_pool()
        .execute("select 1")
        .await
        .is_ok();

    if db_healthy {
        conn.ok("ok!")
    } else {
        conn.with_status(500).halt()
    }
}
