use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Queue::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Queue::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Queue::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Queue::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Queue::ScheduledAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Queue::FailureCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Queue::Status)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Queue::Job).json().not_null())
                    .col(ColumnDef::new(Queue::Result).json().null())
                    .col(ColumnDef::new(Queue::ParentId).uuid().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("index-queue-on-updated-at")
                    .table(Queue::Table)
                    .col(Queue::UpdatedAt)
                    .index_type(IndexType::BTree)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Queue::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Queue {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    ScheduledAt,
    FailureCount,
    Status,
    Job,
    Result,
    ParentId,
}
