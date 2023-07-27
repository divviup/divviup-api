use sea_orm_migration::prelude::*;
use serde_json::json;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.alter_table(
            TableAlterStatement::new()
                .table(Aggregator::Table)
                .add_column(ColumnDef::new(Aggregator::Vdafs).json().null())
                .add_column(ColumnDef::new(Aggregator::QueryTypes).json().null())
                .to_owned(),
        )
        .await?;

        db.exec_stmt(
            Query::update()
                .table(Aggregator::Table)
                .values([
                    (Aggregator::Vdafs, json!([1, 2, 3]).into()), // [VdafId::Prio3Count, VdafId::Prio3Sum, VdafId::Prio3Histogram]
                    (Aggregator::QueryTypes, json!([1, 2]).into()), // [QueryTypeId::TimeInterval, QueryTypeId::FixedSize]
                ])
                .to_owned(),
        )
        .await?;

        db.alter_table(
            TableAlterStatement::new()
                .table(Aggregator::Table)
                .modify_column(ColumnDef::new(Aggregator::Vdafs).not_null())
                .modify_column(ColumnDef::new(Aggregator::QueryTypes).not_null())
                .to_owned(),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.alter_table(
            TableAlterStatement::new()
                .table(Aggregator::Table)
                .drop_column(Aggregator::Vdafs)
                .drop_column(Aggregator::QueryTypes)
                .to_owned(),
        )
        .await?;
        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Aggregator {
    Table,
    Vdafs,
    QueryTypes,
}
