use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(CollectorCredential::Table)
                    .add_column(
                        ColumnDef::new(CollectorCredential::TokenHash)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(CollectorCredential::Table)
                    .rename_column(
                        CollectorCredential::Contents,
                        CollectorCredential::HpkeConfig,
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(CollectorCredential::Table)
                    .rename_column(
                        CollectorCredential::HpkeConfig,
                        CollectorCredential::Contents,
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(CollectorCredential::Table)
                    .drop_column(CollectorCredential::TokenHash)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum CollectorCredential {
    Table,
    Contents,
    TokenHash,
    HpkeConfig,
}
