use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .rename_table(
                TableRenameStatement::new()
                    .table(HpkeConfig::Table, CollectorCredential::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(Task::Table)
                    .rename_column(Task::HpkeConfigId, Task::CollectorCredentialId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(Task::Table)
                    .rename_column(Task::CollectorCredentialId, Task::HpkeConfigId)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                TableRenameStatement::new()
                    .table(CollectorCredential::Table, HpkeConfig::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum HpkeConfig {
    Table,
}

#[derive(DeriveIden)]
enum CollectorCredential {
    Table,
}

#[derive(DeriveIden)]

enum Task {
    Table,
    CollectorCredentialId,
    HpkeConfigId,
}
