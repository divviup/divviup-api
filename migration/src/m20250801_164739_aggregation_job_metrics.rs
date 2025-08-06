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
                        ColumnDef::new(Task::AggregationJobCounterSuccess)
                            .big_integer()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::AggregationJobCounterHelperBatchCollected)
                            .big_integer()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::AggregationJobCounterHelperReportReplayed)
                            .big_integer()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::AggregationJobCounterHelperReportDropped)
                            .big_integer()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::AggregationJobCounterHelperHpkeUnknownConfigId)
                            .big_integer()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::AggregationJobCounterHelperHpkeDecryptFailure)
                            .big_integer()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::AggregationJobCounterHelperVdafPrepError)
                            .big_integer()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::AggregationJobCounterHelperTaskExpired)
                            .big_integer()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::AggregationJobCounterHelperInvalidMessage)
                            .big_integer()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Task::AggregationJobCounterHelperReportTooEarly)
                            .big_integer()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .drop_column(Task::AggregationJobCounterSuccess)
                    .drop_column(Task::AggregationJobCounterHelperBatchCollected)
                    .drop_column(Task::AggregationJobCounterHelperReportReplayed)
                    .drop_column(Task::AggregationJobCounterHelperReportDropped)
                    .drop_column(Task::AggregationJobCounterHelperHpkeUnknownConfigId)
                    .drop_column(Task::AggregationJobCounterHelperHpkeDecryptFailure)
                    .drop_column(Task::AggregationJobCounterHelperVdafPrepError)
                    .drop_column(Task::AggregationJobCounterHelperTaskExpired)
                    .drop_column(Task::AggregationJobCounterHelperInvalidMessage)
                    .drop_column(Task::AggregationJobCounterHelperReportTooEarly)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Task {
    Table,

    AggregationJobCounterSuccess,
    AggregationJobCounterHelperBatchCollected,
    AggregationJobCounterHelperReportReplayed,
    AggregationJobCounterHelperReportDropped,
    AggregationJobCounterHelperHpkeUnknownConfigId,
    AggregationJobCounterHelperHpkeDecryptFailure,
    AggregationJobCounterHelperVdafPrepError,
    AggregationJobCounterHelperTaskExpired,
    AggregationJobCounterHelperInvalidMessage,
    AggregationJobCounterHelperReportTooEarly,
}
