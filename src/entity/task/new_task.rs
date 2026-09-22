use super::*;
use crate::{
    clients::aggregator_client::api_types::AggregatorVdaf,
    entity::{
        aggregator::{Feature, QueryTypeName, Role},
        task, Account, CollectorCredential, Protocol,
    },
    handler::Error,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngExt;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use validator::{ValidationErrors, ValidationErrorsKind};
use vdaf::{DpStrategy, DpStrategyKind, SumVec};

#[derive(Deserialize, Validate, Debug, Clone, Default)]
pub struct NewTask {
    #[validate(required, length(min = 1, max = 255))]
    pub name: Option<String>,

    #[validate(required)]
    pub leader_aggregator_id: Option<String>,

    #[validate(required)]
    pub helper_aggregator_id: Option<String>,

    #[validate(required, nested)]
    pub vdaf: Option<Vdaf>,

    pub query_type: Option<task::QueryType>,

    #[validate(required, range(min = 100))]
    pub min_batch_size: Option<u64>,

    #[validate(range(min = 0))]
    pub max_batch_size: Option<u64>,

    #[validate(range(min = 0))]
    pub batch_time_window_size_seconds: Option<u64>,

    #[validate(
        required,
        range(
            min = 60,
            max = 2592000,
            message = "must be between 1 minute and 4 weeks"
        )
    )]
    pub time_precision_seconds: Option<u64>,

    #[validate(required)]
    pub collector_credential_id: Option<String>,
}

async fn load_aggregator(
    account: &Account,
    id: Option<&str>,
    db: &impl ConnectionTrait,
) -> Result<Option<Aggregator>, Error> {
    let Some(id) = id.map(Uuid::parse_str).transpose()? else {
        return Ok(None);
    };

    let aggregator = Aggregators::find_by_id(id)
        .filter(AggregatorColumn::DeletedAt.is_null())
        .one(db)
        .await?;

    let Some(aggregator) = aggregator else {
        return Ok(None);
    };

    if aggregator.account_id.is_none() || aggregator.account_id == Some(account.id) {
        Ok(Some(aggregator))
    } else {
        Ok(None)
    }
}

const VDAF_BYTES: usize = 16;
fn generate_vdaf_verify_key_and_expected_task_id() -> (String, String) {
    let mut verify_key = [0; VDAF_BYTES];
    rand::rng().fill(&mut verify_key);
    (
        URL_SAFE_NO_PAD.encode(verify_key),
        URL_SAFE_NO_PAD.encode(Sha256::digest(verify_key)),
    )
}

impl NewTask {
    fn validate_min_lte_max(&self, errors: &mut ValidationErrors) {
        let min = self.min_batch_size;
        let max = self.max_batch_size;
        if matches!((min, max), (Some(min), Some(max)) if min > max) {
            let error = ValidationError::new("min_greater_than_max");
            errors.add("min_batch_size", error.clone());
            errors.add("max_batch_size", error);
        }
    }

    fn validate_batch_time_window_size(
        &self,
        query_type: QueryType,
        errors: &mut ValidationErrors,
    ) {
        let window = self.batch_time_window_size_seconds;
        if let Some(window) = window {
            if query_type != QueryType::FixedSize {
                errors.add(
                    "batch_time_window_size_seconds",
                    ValidationError::new("wrong-query-type"),
                );
            };
            if let Some(precision) = self.time_precision_seconds {
                if window % precision != 0 {
                    errors.add(
                        "batch_time_window_size_seconds",
                        ValidationError::new("not-multiple-of-time-precision"),
                    );
                }
            }
        }
    }

    async fn load_collector_credential(
        &self,
        account: &Account,
        db: &impl ConnectionTrait,
    ) -> Option<CollectorCredential> {
        let id = Uuid::parse_str(self.collector_credential_id.as_deref()?).ok()?;
        CollectorCredentials::find_by_id(id)
            .filter(CollectorCredentialColumn::AccountId.eq(account.id))
            .one(db)
            .await
            .ok()
            .flatten()
    }

    async fn validate_collector_credential(
        &self,
        account: &Account,
        leader: Option<&Aggregator>,
        db: &impl ConnectionTrait,
        errors: &mut ValidationErrors,
    ) -> Option<CollectorCredential> {
        match self.load_collector_credential(account, db).await {
            None => {
                errors.add("collector_credential_id", ValidationError::new("required"));
                None
            }

            Some(collector_credential) => {
                let leader_needs_token_hash =
                    leader.is_some_and(|leader| leader.features.token_hash_enabled());

                if leader_needs_token_hash && collector_credential.token_hash.is_none() {
                    errors.add(
                        "collector_credential_id",
                        ValidationError::new("missing-token-hash"),
                    );
                    None
                } else {
                    Some(collector_credential)
                }
            }
        }
    }

    async fn validate_aggregators(
        &self,
        account: &Account,
        db: &impl ConnectionTrait,
        errors: &mut ValidationErrors,
    ) -> Option<(Aggregator, Aggregator, Protocol)> {
        let leader = load_aggregator(account, self.leader_aggregator_id.as_deref(), db)
            .await
            .ok()
            .flatten();
        if leader.is_none() {
            errors.add("leader_aggregator_id", ValidationError::new("required"));
        }

        let helper = load_aggregator(account, self.helper_aggregator_id.as_deref(), db)
            .await
            .ok()
            .flatten();
        if helper.is_none() {
            errors.add("helper_aggregator_id", ValidationError::new("required"));
        }

        let (Some(leader), Some(helper)) = (leader, helper) else {
            return None;
        };

        if leader == helper {
            errors.add("leader_aggregator_id", ValidationError::new("same"));
            errors.add("helper_aggregator_id", ValidationError::new("same"));
        }

        if !leader.is_first_party && !helper.is_first_party {
            errors.add(
                "leader_aggregator_id",
                ValidationError::new("no-first-party"),
            );
            errors.add(
                "helper_aggregator_id",
                ValidationError::new("no-first-party"),
            );
        }

        let resolved_protocol = if leader.protocol == helper.protocol {
            leader.protocol
        } else {
            errors.add("leader_aggregator_id", ValidationError::new("protocol"));
            errors.add("helper_aggregator_id", ValidationError::new("protocol"));
            return None;
        };

        if leader.role == Role::Helper {
            errors.add("leader_aggregator_id", ValidationError::new("role"))
        }

        if helper.role == Role::Leader {
            errors.add("helper_aggregator_id", ValidationError::new("role"))
        }

        if self.batch_time_window_size_seconds.is_some()
            && !leader.features.contains(&Feature::TimeBucketedFixedSize)
        {
            errors.add(
                "leader_aggregator_id",
                ValidationError::new("time-bucketed-fixed-size-unsupported"),
            )
        }

        let uses_pure_dp_discrete_laplace = match &self.vdaf {
            Some(Vdaf::SumVec(SumVec {
                dp_strategy:
                    DpStrategy {
                        dp_strategy: DpStrategyKind::PureDpDiscreteLaplace,
                        ..
                    },
                ..
            })) => true,
            Some(Vdaf::Histogram(histogram)) => matches!(
                histogram.dp_strategy().dp_strategy,
                DpStrategyKind::PureDpDiscreteLaplace
            ),
            _ => false,
        };
        if uses_pure_dp_discrete_laplace
            && !leader.features.contains(&Feature::PureDpDiscreteLaplace)
        {
            errors.add(
                "leader_aggregator_id",
                ValidationError::new("pure-dp-discrete-laplace-unsupported"),
            );
        }
        if uses_pure_dp_discrete_laplace
            && !helper.features.contains(&Feature::PureDpDiscreteLaplace)
        {
            errors.add(
                "helper_aggregator_id",
                ValidationError::new("pure-dp-discrete-laplace-unsupported"),
            );
        }

        if errors.is_empty() {
            Some((leader, helper, resolved_protocol))
        } else {
            None
        }
    }

    fn validate_vdaf_is_supported(
        &self,
        leader: &Aggregator,
        helper: &Aggregator,
        protocol: &Protocol,
        errors: &mut ValidationErrors,
    ) -> Option<AggregatorVdaf> {
        let vdaf = self.vdaf.as_ref()?;

        let name = vdaf.name();
        let aggregator_vdaf = match vdaf.representation_for_protocol(protocol) {
            Ok(vdaf) => vdaf,
            Err(e) => {
                let errors = errors
                    .errors_mut()
                    .entry(Cow::Borrowed("vdaf"))
                    .or_insert_with(|| {
                        ValidationErrorsKind::Struct(Box::new(ValidationErrors::new()))
                    });
                match errors {
                    ValidationErrorsKind::Struct(errors) => {
                        errors.errors_mut().extend(e.into_errors())
                    }
                    other => *other = ValidationErrorsKind::Struct(Box::new(e)),
                };
                return None;
            }
        };

        if !leader.vdafs.contains(&name) || !helper.vdafs.contains(&name) {
            let errors = errors
                .errors_mut()
                .entry(Cow::Borrowed("vdaf"))
                .or_insert_with(|| ValidationErrorsKind::Struct(Box::new(ValidationErrors::new())));
            match errors {
                ValidationErrorsKind::Struct(errors) => {
                    errors.add("type", ValidationError::new("not-supported"));
                }
                other => {
                    let mut e = ValidationErrors::new();
                    e.add("type", ValidationError::new("not-supported"));
                    *other = ValidationErrorsKind::Struct(Box::new(e));
                }
            };
        }

        Some(aggregator_vdaf)
    }

    fn populate_chunk_length(&mut self, protocol: &Protocol) {
        if let Some(vdaf) = &mut self.vdaf {
            vdaf.populate_chunk_length(protocol);
        }
    }

    fn validate_query_type_is_supported(
        &self,
        leader: &Aggregator,
        helper: &Aggregator,
        query_type: &QueryTypeName,
        errors: &mut ValidationErrors,
    ) {
        if !leader.query_types.contains(query_type) || !helper.query_types.contains(query_type) {
            errors.add("query_type", ValidationError::new("not-supported"));
        }
    }

    fn validate_query_type(
        &self,
        query_type: task::QueryType,
        protocol: &Protocol,
        errors: &mut ValidationErrors,
    ) {
        // Note DAP draft-09 says that `max_batch_size` is optional. In DAP draft-04,
        // `max_batch_size` was mandatory for fixed size tasks, and previous versions of divviup-api
        // relied on that linkage.
        match (protocol, query_type, self.max_batch_size) {
            (Protocol::Dap09, QueryType::TimeInterval, None)
            | (Protocol::Dap09, QueryType::FixedSize, Some(_))
            | (Protocol::Dap09, QueryType::FixedSize, None) => {}
            (Protocol::Dap09, QueryType::TimeInterval, Some(_)) => {
                errors.add("max_batch_size", ValidationError::new("conflict"));
            }
        }
    }

    pub async fn normalize_and_validate(
        &mut self,
        account: Account,
        db: &impl ConnectionTrait,
    ) -> Result<ProvisionableTask, ValidationErrors> {
        let mut errors = Validate::validate(self).err().unwrap_or_default();

        // Backfill the query type from `max_batch_size` if necessary.
        //
        // This was previously not part of requests. For the initial support of DAP draft-04, the
        // query type was inferred from the presence or absence of a `max_batch_size` value.
        let query_type = self.query_type.unwrap_or(match self.max_batch_size {
            Some(_) => task::QueryType::FixedSize,
            None => task::QueryType::TimeInterval,
        });
        let query_type_name = QueryTypeName::from(query_type);

        self.validate_min_lte_max(&mut errors);
        self.validate_batch_time_window_size(query_type, &mut errors);
        let aggregators = self.validate_aggregators(&account, db, &mut errors).await;
        let collector_credential = self
            .validate_collector_credential(
                &account,
                aggregators.as_ref().map(|(leader, ..)| leader),
                db,
                &mut errors,
            )
            .await;

        let aggregator_vdaf = if let Some((leader, helper, protocol)) = aggregators.as_ref() {
            self.validate_query_type(query_type, protocol, &mut errors);
            self.validate_query_type_is_supported(leader, helper, &query_type_name, &mut errors);
            self.populate_chunk_length(protocol);
            self.validate_vdaf_is_supported(leader, helper, protocol, &mut errors)
        } else {
            None
        };

        if errors.is_empty() {
            // Unwrap safety: All of these unwraps below have previously
            // been checked by the above validations. The fact that we
            // have to check them twice is a consequence of the
            // disharmonious combination of Validate and the fact that we
            // need to use options for all fields so serde doesn't bail on
            // the first error.
            let (leader_aggregator, helper_aggregator, protocol) = aggregators.unwrap();

            let (vdaf_verify_key, id) = generate_vdaf_verify_key_and_expected_task_id();

            Ok(ProvisionableTask {
                account,
                id,
                vdaf_verify_key,
                name: self.name.clone().unwrap(),
                leader_aggregator,
                helper_aggregator,
                vdaf: self.vdaf.clone().unwrap(),
                query_type,
                aggregator_vdaf: aggregator_vdaf.unwrap(),
                min_batch_size: self.min_batch_size.unwrap(),
                max_batch_size: self.max_batch_size,
                batch_time_window_size_seconds: self.batch_time_window_size_seconds,
                expiration: Some(OffsetDateTime::now_utc() + DEFAULT_EXPIRATION_DURATION),
                time_precision_seconds: self.time_precision_seconds.unwrap(),
                collector_credential: collector_credential.unwrap(),
                aggregator_auth_token: None,
                protocol,
            })
        } else {
            Err(errors)
        }
    }
}
