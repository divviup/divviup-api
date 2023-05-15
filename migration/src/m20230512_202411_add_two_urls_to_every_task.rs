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
                    .add_column(ColumnDef::new(Task::LeaderUrl).string().null())
                    .add_column(ColumnDef::new(Task::HelperUrl).string().null())
                    .drop_column(Task::Partner)
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::update()
                    .table(Task::Table)
                    .values([
                        (Task::LeaderUrl, "https://leader.test".into()),
                        (Task::HelperUrl, "https://helper.test".into()),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .modify_column(ColumnDef::new(Task::LeaderUrl).string().not_null())
                    .modify_column(ColumnDef::new(Task::HelperUrl).string().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .add_column(
                        ColumnDef::new(Task::Partner)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .drop_column(Task::HelperUrl)
                    .drop_column(Task::LeaderUrl)
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
