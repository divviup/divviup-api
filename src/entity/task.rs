use crate::{
    clients::aggregator_client::{
        api_types::{Role, TaskResponse},
        TaskMetrics,
    },
    entity::{account, membership, url::Url, Account},
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

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "task")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub account_id: Uuid,
    pub name: String,
    pub leader_url: Url,
    pub helper_url: Url,
    pub vdaf: Vdaf,
    pub min_batch_size: i64,
    pub max_batch_size: Option<i64>,
    pub is_leader: bool,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
    pub time_precision_seconds: i32,
    pub report_count: i32,
    pub aggregate_collection_count: i32,
    #[serde(default, with = "time::serde::iso8601::option")]
    pub expiration: Option<OffsetDateTime>,
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
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Account,
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

pub fn build_task(mut task: NewTask, api_response: TaskResponse, account: &Account) -> ActiveModel {
    ActiveModel {
        id: Set(api_response.task_id.to_string()),
        account_id: Set(account.id),
        name: Set(task.name.take().unwrap()),
        leader_url: Set(api_response.leader_endpoint.clone().into()),
        helper_url: Set(api_response.helper_endpoint.clone().into()),
        vdaf: Set(Vdaf::from(api_response.vdaf)),
        min_batch_size: Set(api_response.min_batch_size.try_into().unwrap()),
        max_batch_size: Set(api_response.query_type.into()),
        is_leader: Set(matches!(api_response.role, Role::Leader)),
        created_at: Set(OffsetDateTime::now_utc()),
        updated_at: Set(OffsetDateTime::now_utc()),
        time_precision_seconds: Set(api_response.time_precision.as_seconds().try_into().unwrap()),
        report_count: Set(0),
        aggregate_collection_count: Set(0),
        expiration: Set(api_response.task_expiration.map(|t| {
            OffsetDateTime::from_unix_timestamp(t.as_seconds_since_epoch().try_into().unwrap())
                .unwrap()
        })),
    }
}
