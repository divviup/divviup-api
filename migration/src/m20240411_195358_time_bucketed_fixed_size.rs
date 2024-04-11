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
                    .add_column(
                        ColumnDef::new(Task::BatchTimeWindowSize)
                            .big_integer()
                            .null(),
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
                    .drop_column(Task::BatchTimeWindowSize)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Task {
    Table,
    BatchTimeWindowSize,
}
