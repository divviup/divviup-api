use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .exec_stmt(Table::truncate().table(Task::Table).to_owned())
            .await?;
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(Task::Table)
                    .add_column(ColumnDef::new(Task::LeaderAggregatorId).uuid().not_null())
                    .add_column(ColumnDef::new(Task::HelperAggregatorId).uuid().not_null())
                    .drop_column(Task::IsLeader)
                    .drop_column(Task::HelperUrl)
                    .drop_column(Task::LeaderUrl)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .exec_stmt(Table::truncate().table(Task::Table).to_owned())
            .await?;
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(Task::Table)
                    .drop_column(Task::LeaderAggregatorId)
                    .drop_column(Task::HelperAggregatorId)
                    .add_column(
                        ColumnDef::new(Task::IsLeader)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(ColumnDef::new(Task::LeaderUrl).string().not_null())
                    .add_column(ColumnDef::new(Task::HelperUrl).string().not_null())
                    .to_owned(),
            )
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Task {
    Table,
    LeaderAggregatorId,
    HelperAggregatorId,
    IsLeader,
    LeaderUrl,
    HelperUrl,
}
