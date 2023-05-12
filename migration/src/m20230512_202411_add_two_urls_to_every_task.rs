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
                    .add_column(ColumnDef::new(Task::LeaderUrl).string().not_null())
                    .add_column(ColumnDef::new(Task::HelperUrl).string().not_null())
                    .remove_column(ColumnDef::new(Task::Partner).string().not_null())
                    .to_owned(),
            )
            .await?;
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .add_column(ColumnDef::new(Task::Partner).string().not_null())
                    .remove_column(ColumnDef::new(Task::HelperUrl).string().not_null())
                    .remove_column(ColumnDef::new(Task::LeaderUrl).string().not_null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Task {
    Table,
    Partner,
    LeaderUrl,
    HelperUrl,
}
