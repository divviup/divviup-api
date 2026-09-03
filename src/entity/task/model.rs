use crate::{
    clients::aggregator_client::{api_types::TaskAggregationJobMetrics, TaskUploadMetrics},
    entity::{
        account, json::Json, membership, AccountColumn, Accounts, Aggregator, AggregatorColumn,
        Aggregators, CollectorCredentialColumn, CollectorCredentials,
    },
};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ActiveValue, ConnectionTrait, DbErr, DeriveEntityModel,
    DerivePrimaryKey, DeriveRelation, EntityTrait, EnumIter, IntoActiveModel, PrimaryKeyTrait,
    Related, RelationDef, RelationTrait,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use time::OffsetDateTime;
use uuid::Uuid;

use super::vdaf::Vdaf;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "task")]
pub struct Model {
    /// The DAP task ID.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub account_id: Uuid,
    pub name: String,
    pub vdaf: Json<Vdaf>,
    pub min_batch_size: i64,
    pub max_batch_size: Option<i64>,
    pub batch_time_window_size_seconds: Option<i64>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
    pub time_precision_seconds: i32,

    /// Deprecated metrics field. Never populated, only reads zero.
    #[sea_orm(ignore)]
    #[serde(default)]
    pub report_count: i32,
    /// Deprecated metrics field. Never populated, only reads zero.
    #[sea_orm(ignore)]
    #[serde(default)]
    pub aggregate_collection_count: i32,

    #[serde(default, with = "time::serde::rfc3339::option")]
    pub expiration: Option<OffsetDateTime>,
    pub leader_aggregator_id: Uuid,
    pub helper_aggregator_id: Uuid,
    pub collector_credential_id: Uuid,

    // Report upload metrics
    pub report_counter_interval_collected: i64,
    pub report_counter_decode_failure: i64,
    pub report_counter_decrypt_failure: i64,
    pub report_counter_expired: i64,
    pub report_counter_outdated_key: i64,
    pub report_counter_success: i64,
    pub report_counter_too_early: i64,
    pub report_counter_task_expired: i64,
    pub report_counter_duplicate_extension: i64,

    // Aggregation job metrics
    pub aggregation_job_counter_success: i64,
    pub aggregation_job_counter_helper_batch_collected: i64,
    pub aggregation_job_counter_helper_report_replayed: i64,
    pub aggregation_job_counter_helper_report_dropped: i64,
    pub aggregation_job_counter_helper_hpke_unknown_config_id: i64,
    pub aggregation_job_counter_helper_hpke_decrypt_failure: i64,
    pub aggregation_job_counter_helper_vdaf_prep_error: i64,
    pub aggregation_job_counter_helper_task_expired: i64,
    pub aggregation_job_counter_helper_invalid_message: i64,
    pub aggregation_job_counter_helper_report_too_early: i64,
}

impl Model {
    pub async fn update_task_upload_metrics(
        self,
        metrics: TaskUploadMetrics,
        db: impl ConnectionTrait,
    ) -> Result<Self, DbErr> {
        let mut task = self.into_active_model();
        task.report_counter_interval_collected =
            ActiveValue::Set(metrics.interval_collected.try_into().unwrap_or(i64::MAX));
        task.report_counter_decode_failure =
            ActiveValue::Set(metrics.report_decode_failure.try_into().unwrap_or(i64::MAX));
        task.report_counter_decrypt_failure = ActiveValue::Set(
            metrics
                .report_decrypt_failure
                .try_into()
                .unwrap_or(i64::MAX),
        );
        task.report_counter_expired =
            ActiveValue::Set(metrics.report_expired.try_into().unwrap_or(i64::MAX));
        task.report_counter_outdated_key =
            ActiveValue::Set(metrics.report_outdated_key.try_into().unwrap_or(i64::MAX));
        task.report_counter_success =
            ActiveValue::Set(metrics.report_success.try_into().unwrap_or(i64::MAX));
        task.report_counter_too_early =
            ActiveValue::Set(metrics.report_too_early.try_into().unwrap_or(i64::MAX));
        task.report_counter_task_expired =
            ActiveValue::Set(metrics.task_expired.try_into().unwrap_or(i64::MAX));
        task.report_counter_duplicate_extension = ActiveValue::Set(
            metrics
                .report_duplicate_extension
                .try_into()
                .unwrap_or(i64::MAX),
        );
        task.updated_at = ActiveValue::Set(OffsetDateTime::now_utc());
        task.update(&db).await
    }

    pub async fn update_task_aggregation_job_metrics(
        self,
        metrics: TaskAggregationJobMetrics,
        db: impl ConnectionTrait,
    ) -> Result<Self, DbErr> {
        let mut task = self.into_active_model();
        task.aggregation_job_counter_success =
            ActiveValue::Set(metrics.success.try_into().unwrap_or(i64::MAX));
        task.aggregation_job_counter_helper_batch_collected = ActiveValue::Set(
            metrics
                .helper_batch_collected
                .try_into()
                .unwrap_or(i64::MAX),
        );
        task.aggregation_job_counter_helper_report_replayed = ActiveValue::Set(
            metrics
                .helper_report_replayed
                .try_into()
                .unwrap_or(i64::MAX),
        );
        task.aggregation_job_counter_helper_report_dropped =
            ActiveValue::Set(metrics.helper_report_dropped.try_into().unwrap_or(i64::MAX));
        task.aggregation_job_counter_helper_hpke_unknown_config_id = ActiveValue::Set(
            metrics
                .helper_hpke_unknown_config_id
                .try_into()
                .unwrap_or(i64::MAX),
        );
        task.aggregation_job_counter_helper_hpke_decrypt_failure = ActiveValue::Set(
            metrics
                .helper_hpke_decrypt_failure
                .try_into()
                .unwrap_or(i64::MAX),
        );
        task.aggregation_job_counter_helper_vdaf_prep_error = ActiveValue::Set(
            metrics
                .helper_vdaf_prep_error
                .try_into()
                .unwrap_or(i64::MAX),
        );
        task.aggregation_job_counter_helper_task_expired =
            ActiveValue::Set(metrics.helper_task_expired.try_into().unwrap_or(i64::MAX));
        task.aggregation_job_counter_helper_invalid_message = ActiveValue::Set(
            metrics
                .helper_invalid_message
                .try_into()
                .unwrap_or(i64::MAX),
        );
        task.aggregation_job_counter_helper_report_too_early = ActiveValue::Set(
            metrics
                .helper_report_too_early
                .try_into()
                .unwrap_or(i64::MAX),
        );
        task.updated_at = ActiveValue::Set(OffsetDateTime::now_utc());
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

    pub async fn helper_aggregator(&self, db: &impl ConnectionTrait) -> Result<Aggregator, DbErr> {
        Aggregators::find_by_id(self.helper_aggregator_id)
            .one(db)
            .await
            .transpose()
            .ok_or(DbErr::Custom("expected helper aggregator".into()))?
    }

    pub async fn aggregators(&self, db: &impl ConnectionTrait) -> Result<[Aggregator; 2], DbErr> {
        futures_lite::future::try_zip(self.leader_aggregator(db), self.helper_aggregator(db))
            .await
            .map(|(leader, helper)| [leader, helper])
    }

    pub async fn first_party_aggregator(
        &self,
        db: &impl ConnectionTrait,
    ) -> Result<Option<Aggregator>, DbErr> {
        Ok(self
            .aggregators(db)
            .await?
            .into_iter()
            .find(|agg| agg.is_first_party))
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Accounts",
        from = "Column::AccountId",
        to = "AccountColumn::Id"
    )]
    Account,

    #[sea_orm(
        belongs_to = "Aggregators",
        from = "Column::HelperAggregatorId",
        to = "AggregatorColumn::Id"
    )]
    HelperAggregator,

    #[sea_orm(
        belongs_to = "Aggregators",
        from = "Column::LeaderAggregatorId",
        to = "AggregatorColumn::Id"
    )]
    LeaderAggregator,

    #[sea_orm(
        belongs_to = "CollectorCredentials",
        from = "Column::CollectorCredentialId",
        to = "CollectorCredentialColumn::Id"
    )]
    CollectorCredential,
}

impl Related<account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<membership::Entity> for Entity {
    fn to() -> RelationDef {
        account::Relation::Memberships.def()
    }

    fn via() -> Option<RelationDef> {
        Some(account::Relation::Tasks.def().rev())
    }
}

impl Related<CollectorCredentials> for Entity {
    fn to() -> RelationDef {
        Relation::CollectorCredential.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
