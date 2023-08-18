use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.alter_table(
            TableAlterStatement::new()
                .table(Aggregator::Table)
                .add_column(ColumnDef::new(Aggregator::Protocol).string().null())
                .to_owned(),
        )
        .await?;

        db.exec_stmt(
            Query::update()
                .table(Aggregator::Table)
                .value(Aggregator::Protocol, "DAP-04")
                .to_owned(),
        )
        .await?;

        db.alter_table(
            TableAlterStatement::new()
                .table(Aggregator::Table)
                .modify_column(ColumnDef::new(Aggregator::Protocol).not_null())
                .to_owned(),
        )
        .await?;

        db.alter_table(
            TableAlterStatement::new()
                .table(Task::Table)
                .add_column(ColumnDef::new(Task::Protocol).string().null())
                .to_owned(),
        )
        .await?;

        db.exec_stmt(
            Query::update()
                .table(Task::Table)
                .value(Task::Protocol, "DAP-04")
                .to_owned(),
        )
        .await?;

        db.alter_table(
            TableAlterStatement::new()
                .table(Task::Table)
                .modify_column(ColumnDef::new(Task::Protocol).not_null())
                .to_owned(),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.alter_table(
            TableAlterStatement::new()
                .table(Aggregator::Table)
                .drop_column(Aggregator::Protocol)
                .to_owned(),
        )
        .await?;
        db.alter_table(
            TableAlterStatement::new()
                .table(Task::Table)
                .drop_column(Task::Protocol)
                .to_owned(),
        )
        .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Aggregator {
    Table,
    Protocol,
}

#[derive(DeriveIden)]
enum Task {
    Table,
    Protocol,
}
