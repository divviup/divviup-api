use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Task::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Task::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Task::AccountId).uuid().not_null())
                    .col(ColumnDef::new(Task::Name).string().not_null())
                    .col(ColumnDef::new(Task::Partner).string().not_null())
                    .col(ColumnDef::new(Task::Vdaf).json().not_null())
                    .col(ColumnDef::new(Task::MinBatchSize).big_integer().not_null())
                    .col(
                        ColumnDef::new(Task::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Task::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Task::TimePrecisionSeconds)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Task::ReportCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Task::AggregateCollectionCount)
                            .integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Task::Expiration).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("index-task-account-id")
                    .table(Task::Table)
                    .col(Task::AccountId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("index-task-account-id")
                    .table(Task::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Task::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Task {
    Table,
    Id,
    Name,
    Partner,
    Vdaf,
    MinBatchSize,
    TimePrecisionSeconds,
    ReportCount,
    AggregateCollectionCount,
    AccountId,
    CreatedAt,
    UpdatedAt,
    Expiration,
}
