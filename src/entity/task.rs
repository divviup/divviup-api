use crate::{
    clients::aggregator_client::{api_types::TaskResponse, TaskMetrics},
    entity::{account, membership},
};
use sea_orm::{entity::prelude::*, ActiveValue::Set, IntoActiveModel};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use validator::{Validate, ValidationError};

pub mod vdaf;
use vdaf::Vdaf;
mod new_task;
pub use new_task::NewTask;
mod update_task;
pub use update_task::UpdateTask;
mod provisionable_task;
pub use provisionable_task::{ProvisionableTask, TaskProvisioningError};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "task")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub account_id: Uuid,
    pub name: String,
    pub vdaf: Vdaf,
    pub min_batch_size: i64,
    pub max_batch_size: Option<i64>,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
    pub time_precision_seconds: i32,
    pub report_count: i32,
    pub aggregate_collection_count: i32,
    #[serde(default, with = "time::serde::iso8601::option")]
    pub expiration: Option<OffsetDateTime>,
    pub leader_aggregator_id: Uuid,
    pub helper_aggregator_id: Uuid,
}

impl Model {
    pub async fn update_metrics(
        self,
        metrics: TaskMetrics,
        db: impl ConnectionTrait,
    ) -> Result<Self, DbErr> {
        let mut task = self.into_active_model();
        task.report_count = Set(metrics.reports.try_into().unwrap_or(i32::MAX));
        task.aggregate_collection_count =
            Set(metrics.report_aggregations.try_into().unwrap_or(i32::MAX));
        task.updated_at = Set(OffsetDateTime::now_utc());
        task.update(&db).await
    }

    pub async fn leader_aggregator(
        &self,
        db: &impl ConnectionTrait,
    ) -> Result<super::Aggregator, DbErr> {
        super::Aggregators::find_by_id(self.leader_aggregator_id)
            .one(db)
            .await
            .transpose()
            .ok_or(DbErr::Custom("expected leader aggregator".into()))?
    }

    pub async fn helper_aggregator(
        &self,
        db: &impl ConnectionTrait,
    ) -> Result<super::Aggregator, DbErr> {
        super::Aggregators::find_by_id(self.leader_aggregator_id)
            .one(db)
            .await
            .transpose()
            .ok_or(DbErr::Custom("expected helper aggregator".into()))?
    }

    pub async fn aggregators(
        &self,
        db: &impl ConnectionTrait,
    ) -> Result<[super::Aggregator; 2], DbErr> {
        let (leader, helper) =
            futures_lite::future::try_zip(self.leader_aggregator(db), self.helper_aggregator(db))
                .await?;
        Ok([leader, helper])
    }

    pub async fn first_party_aggregator(
        &self,
        db: &impl ConnectionTrait,
    ) -> Result<Option<super::Aggregator>, DbErr> {
        Ok(self
            .aggregators(db)
            .await?
            .into_iter()
            .find(|agg| agg.is_first_party()))
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::Accounts",
        from = "Column::AccountId",
        to = "super::account::Column::Id"
    )]
    Account,

    #[sea_orm(
        belongs_to = "super::Aggregators",
        from = "Column::HelperAggregatorId",
        to = "super::AggregatorColumn::Id"
    )]
    HelperAggregator,

    #[sea_orm(
        belongs_to = "super::Aggregators",
        from = "Column::LeaderAggregatorId",
        to = "super::AggregatorColumn::Id"
    )]
    LeaderAggregator,
}

impl Related<account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<membership::Entity> for Entity {
    fn to() -> RelationDef {
        account::Relation::Membership.def()
    }

    fn via() -> Option<RelationDef> {
        Some(account::Relation::Task.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
