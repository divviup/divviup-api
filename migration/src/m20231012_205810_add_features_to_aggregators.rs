use sea_orm_migration::prelude::*;
use serde_json::json;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Aggregator::Table)
                    .add_column(ColumnDef::new(Aggregator::Features).json().null())
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::update()
                    .table(Aggregator::Table)
                    .value(Aggregator::Features, json!([]))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Aggregator::Table)
                    .modify_column(ColumnDef::new(Aggregator::Features).json().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Aggregator::Table)
                    .drop_column(Aggregator::Features)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Aggregator {
    Table,
    Features,
}
