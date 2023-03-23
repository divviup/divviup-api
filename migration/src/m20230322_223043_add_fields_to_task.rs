use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(Task::Table)
                    .add_column(ColumnDef::new(Task::MaxBatchSize).integer().null())
                    .add_column(
                        ColumnDef::new(Task::IsLeader)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(Task::Table)
                    .drop_column(Task::MaxBatchSize)
                    .drop_column(Task::IsLeader)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Task {
    Table,
    MaxBatchSize,
    IsLeader,
}
