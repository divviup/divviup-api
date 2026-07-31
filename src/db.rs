use log::LevelFilter;
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DbBackend, DbConn, DbErr, ExecResult, QueryResult,
    Statement,
};
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
    fn get_database_backend(&self) -> DbBackend {
        self.0.get_database_backend()
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        self.0.execute_raw(stmt).await
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        self.0.execute_unprepared(sql).await
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        self.0.query_one_raw(stmt).await
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        self.0.query_all_raw(stmt).await
    }
}
