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
                    .add_column(
                        ColumnDef::new(Task::ReportCounterIntervalCollected)
                            .big_integer()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::ReportCounterDecodeFailure)
                            .big_integer()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::ReportCounterDecryptFailure)
                            .big_integer()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::ReportCounterExpired)
                            .big_integer()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::ReportCounterOutdatedKey)
                            .big_integer()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::ReportCounterSuccess)
                            .big_integer()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::ReportCounterTooEarly)
                            .big_integer()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::ReportCounterTaskExpired)
                            .big_integer()
                            .default(0),
                    )
                    .drop_column(Task::ReportCount)
                    .drop_column(Task::AggregateCollectionCount)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .drop_column(Task::ReportCounterIntervalCollected)
                    .drop_column(Task::ReportCounterDecodeFailure)
                    .drop_column(Task::ReportCounterDecryptFailure)
                    .drop_column(Task::ReportCounterExpired)
                    .drop_column(Task::ReportCounterOutdatedKey)
                    .drop_column(Task::ReportCounterSuccess)
                    .drop_column(Task::ReportCounterTooEarly)
                    .drop_column(Task::ReportCounterTaskExpired)
                    // These columns cache transient state. We don't have to worry about refilling
                    // them if we have to execute a down migration, since we'll fetch them from
                    // the source again eventually.
                    .add_column(
                        ColumnDef::new(Task::ReportCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::AggregateCollectionCount)
                            .integer()
                            .default(0)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Task {
    Table,

    ReportCounterIntervalCollected,
    ReportCounterDecodeFailure,
    ReportCounterDecryptFailure,
    ReportCounterExpired,
    ReportCounterOutdatedKey,
    ReportCounterSuccess,
    ReportCounterTooEarly,
    ReportCounterTaskExpired,

    ReportCount,
    AggregateCollectionCount,
}
