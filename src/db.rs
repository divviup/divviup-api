use log::LevelFilter;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DbConn};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug)]
pub struct Db(DbConn);

impl Db {
    pub async fn connect(url: &str) -> Self {
        let mut connect_options = ConnectOptions::new(url);
        connect_options.sqlx_logging_level(LevelFilter::Debug);
        Database::connect(connect_options).await.map(Self).unwrap()
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

#[async_trait::async_trait]
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
