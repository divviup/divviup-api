use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Membership::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Membership::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Membership::AccountId).uuid().not_null())
                    .col(ColumnDef::new(Membership::UserEmail).string().not_null())
                    .col(
                        ColumnDef::new(Membership::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            .name("index-membership-account-user-unique")
                            .table(Membership::Table)
                            .col(Membership::UserEmail)
                            .col(Membership::AccountId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fkey-membership-account-id")
                            .from(Membership::Table, Membership::AccountId)
                            .to(Account::Table, Account::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("index-membership-user-email")
                    .table(Membership::Table)
                    .col(Membership::UserEmail)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Membership::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Membership {
    Table,
    Id,
    UserEmail,
    AccountId,
    CreatedAt,
}

#[derive(Iden)]
enum Account {
    Table,
    Id,
}
