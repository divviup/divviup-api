use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(ApiToken::Table)
                    .add_column(ColumnDef::new(ApiToken::Name).string().null())
                    .add_column(
                        ColumnDef::new(ApiToken::LastUsedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(ApiToken::UpdatedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::update()
                    .table(ApiToken::Table)
                    .value(ApiToken::UpdatedAt, time::OffsetDateTime::now_utc())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(ApiToken::Table)
                    .modify_column(
                        ColumnDef::new(ApiToken::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(ApiToken::Table)
                    .drop_column(ApiToken::Name)
                    .drop_column(ApiToken::LastUsedAt)
                    .drop_column(ApiToken::UpdatedAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum ApiToken {
    Table,
    Name,
    LastUsedAt,
    UpdatedAt,
}
