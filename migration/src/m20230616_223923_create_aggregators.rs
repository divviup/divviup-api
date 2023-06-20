use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Aggregator::Table)
                    .col(
                        ColumnDef::new(Aggregator::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Aggregator::Name).string().not_null())
                    .col(ColumnDef::new(Aggregator::ApiUrl).string().not_null())
                    .col(ColumnDef::new(Aggregator::BearerToken).string().not_null())
                    .col(ColumnDef::new(Aggregator::DapUrl).string().not_null())
                    .col(ColumnDef::new(Aggregator::Role).integer().not_null())
                    .col(ColumnDef::new(Aggregator::AccountId).uuid().null())
                    .col(
                        ColumnDef::new(Aggregator::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Aggregator::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Aggregator::DeletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fkey-aggregator-account-id")
                    .from(Aggregator::Table, Aggregator::AccountId)
                    .to(Account::Table, Account::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Aggregator::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Aggregator {
    Table,
    Id,
    ApiUrl,
    BearerToken,
    DapUrl,
    Role,
    Name,
    AccountId,
    CreatedAt,
    DeletedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Account {
    Table,
    Id,
}
