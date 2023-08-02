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
            .exec_stmt(Table::truncate().table(Aggregator::Table).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Aggregator::Table)
                    .add_column(
                        ColumnDef::new(Aggregator::EncryptedBearerToken)
                            .binary()
                            .not_null(),
                    )
                    .drop_column(Aggregator::BearerToken)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .exec_stmt(Table::truncate().table(Task::Table).to_owned())
            .await?;
        manager
            .exec_stmt(Table::truncate().table(Aggregator::Table).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Aggregator::Table)
                    .add_column(ColumnDef::new(Aggregator::BearerToken).string().not_null())
                    .drop_column(Aggregator::EncryptedBearerToken)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Aggregator {
    Table,
    BearerToken,
    EncryptedBearerToken,
}

#[derive(Iden)]
enum Task {
    Table,
}
