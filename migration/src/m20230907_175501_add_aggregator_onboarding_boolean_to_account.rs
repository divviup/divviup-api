use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.alter_table(
            TableAlterStatement::new()
                .table(Account::Table)
                .add_column(
                    ColumnDef::new(Account::IntendsToUseSharedAggregators)
                        .boolean()
                        .not_null()
                        .default(false),
                )
                .to_owned(),
        )
        .await?;

        db.get_connection()
            .execute_unprepared(
                "UPDATE account
                     SET intends_to_use_shared_aggregators = TRUE
                     WHERE NOT EXISTS (
                         SELECT aggregator.id
                         FROM aggregator
                         WHERE aggregator.account_id = account.id
                         LIMIT 1
                     ) AND EXISTS (
                         SELECT task.id
                         FROM task
                         WHERE task.account_id = account.id
                         LIMIT 1
                     )",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.alter_table(
            TableAlterStatement::new()
                .table(Account::Table)
                .drop_column(Account::IntendsToUseSharedAggregators)
                .to_owned(),
        )
        .await
    }
}

#[derive(DeriveIden)]
enum Account {
    Table,
    IntendsToUseSharedAggregators,
}
