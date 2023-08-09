use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.exec_stmt(Table::truncate().table(Task::Table).to_owned())
            .await?;

        db.create_table(
            Table::create()
                .table(HpkeConfig::Table)
                .col(
                    ColumnDef::new(HpkeConfig::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(HpkeConfig::Contents).binary().not_null())
                .col(ColumnDef::new(HpkeConfig::AccountId).uuid().not_null())
                .col(ColumnDef::new(HpkeConfig::Name).string().null())
                .col(
                    ColumnDef::new(HpkeConfig::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(HpkeConfig::DeletedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(HpkeConfig::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .to_owned(),
        )
        .await?;

        db.alter_table(
            TableAlterStatement::new()
                .table(Task::Table)
                .add_column(ColumnDef::new(Task::HpkeConfigId).uuid().null())
                .to_owned(),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.alter_table(
            TableAlterStatement::new()
                .table(Task::Table)
                .drop_column(Task::HpkeConfigId)
                .to_owned(),
        )
        .await?;

        db.drop_table(Table::drop().table(HpkeConfig::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum HpkeConfig {
    Table,
    Id,
    Contents,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Name,
    AccountId,
}

#[derive(Iden)]
enum Task {
    Table,
    HpkeConfigId,
}
