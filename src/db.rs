use sea_orm::{ConnectionTrait, Database, DbConn};
use std::ops::{Deref, DerefMut};
use trillium::{async_trait, Conn, Handler};
use trillium_api::FromConn;

#[derive(Clone, Debug)]
pub struct Db(DbConn);

impl Db {
    pub async fn connect(url: &str) -> Self {
        Database::connect(url).await.map(Self).unwrap()
    }
}

impl From<DbConn> for Db {
    fn from(value: DbConn) -> Self {
        Self(value)
    }
}

#[async_trait]
impl FromConn for Db {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        conn.state().cloned()
    }
}

#[async_trait]
impl Handler for Db {
    async fn run(&self, conn: Conn) -> Conn {
        conn.with_state(self.clone())
    }
}

impl Deref for Db {
    type Target = DbConn;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Db {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait]
impl ConnectionTrait for Db {
    fn get_database_backend(&self) -> sea_orm::DbBackend {
        self.0.get_database_backend()
    }

    async fn execute(
        &self,
        stmt: sea_orm::Statement,
    ) -> Result<sea_orm::ExecResult, sea_orm::DbErr> {
        self.0.execute(stmt).await
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<sea_orm::ExecResult, sea_orm::DbErr> {
        self.0.execute_unprepared(sql).await
    }

    async fn query_one(
        &self,
        stmt: sea_orm::Statement,
    ) -> Result<Option<sea_orm::QueryResult>, sea_orm::DbErr> {
        self.0.query_one(stmt).await
    }

    async fn query_all(
        &self,
        stmt: sea_orm::Statement,
    ) -> Result<Vec<sea_orm::QueryResult>, sea_orm::DbErr> {
        self.0.query_all(stmt).await
    }
}
