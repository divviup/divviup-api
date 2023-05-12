use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .modify_column(ColumnDef::new(Task::MaxBatchSize).big_integer().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .modify_column(ColumnDef::new(Task::MaxBatchSize).integer().null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Task {
    Table,
    MaxBatchSize,
}
