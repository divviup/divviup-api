use sea_orm::{sea_query::extension::postgres::Type, DbBackend};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // Use an enum in Postgres, and a string in SQLite.
        let mut column_def = if db.get_database_backend() == DbBackend::Postgres {
            manager
                .create_type(
                    Type::create()
                        .as_enum(QueryType::Enum)
                        .values([QueryType::TimeInterval, QueryType::FixedSize])
                        .to_owned(),
                )
                .await?;
            ColumnDef::new(Task::QueryType)
                .custom(QueryType::Enum)
                .to_owned()
        } else {
            ColumnDef::new(Task::QueryType).string().to_owned()
        };
        // Add the column as a nullable column.
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .add_column(column_def.null().default(Expr::null()))
                    .to_owned(),
            )
            .await?;
        // Backfill the column.
        manager
            .execute(
                Query::update()
                    .table(Task::Table)
                    .value(
                        Task::QueryType,
                        Expr::case(
                            Expr::column(Task::MaxBatchSize).is_not_null(),
                            Expr::cast_as(
                                Expr::value(QueryType::FixedSize.unquoted()),
                                QueryType::Enum,
                            ),
                        )
                        .finally(Expr::cast_as(
                            Expr::value(QueryType::TimeInterval.unquoted()),
                            QueryType::Enum,
                        )),
                    )
                    .to_owned(),
            )
            .await?;
        // Change the column to be not nullable.
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .modify_column(column_def.not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .drop_column(Task::QueryType)
                    .to_owned(),
            )
            .await?;
        let db = manager.get_connection();
        if db.get_database_backend() == DbBackend::Postgres {
            manager
                .drop_type(Type::drop().name(QueryType::Enum).to_owned())
                .await?;
        }
        Ok(())
    }
}

#[derive(Iden)]
enum Task {
    Table,

    MaxBatchSize,
    QueryType,
}

#[derive(Iden)]
pub enum QueryType {
    #[iden = "query_type"]
    Enum,

    #[iden = "TIME_INTERVAL"]
    TimeInterval,
    #[iden = "FIXED_SIZE"]
    FixedSize,
}
