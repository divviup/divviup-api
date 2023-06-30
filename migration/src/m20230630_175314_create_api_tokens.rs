use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ApiToken::Table)
                    .col(ColumnDef::new(ApiToken::Id).primary_key().uuid().not_null())
                    .col(ColumnDef::new(ApiToken::AccountId).uuid().not_null())
                    .col(ColumnDef::new(ApiToken::TokenHash).binary().not_null())
                    .col(
                        ColumnDef::new(ApiToken::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ApiToken::DeletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ApiToken::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ApiToken {
    Table,
    Id,
    AccountId,
    TokenHash,
    CreatedAt,
    DeletedAt,
}
