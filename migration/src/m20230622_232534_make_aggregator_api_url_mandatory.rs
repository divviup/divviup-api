use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Aggregator::Table)
                    .modify_column(ColumnDef::new(Aggregator::ApiUrl).string().not_null())
                    .modify_column(ColumnDef::new(Aggregator::BearerToken).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Aggregator::Table)
                    .modify_column(ColumnDef::new(Aggregator::ApiUrl).string().null())
                    .modify_column(ColumnDef::new(Aggregator::BearerToken).string().null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Aggregator {
    Table,
    ApiUrl,
    BearerToken,
}
