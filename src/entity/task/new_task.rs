use super::*;
use crate::{
    clients::aggregator_client::api_types::{AggregatorVdaf, QueryType},
    entity::{
        aggregator::Role, Account, Aggregator, Aggregators, CollectorCredential,
        CollectorCredentialColumn, Protocol,
    },
    handler::Error,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use sea_orm::{ColumnTrait, QueryFilter};
use sha2::{Digest, Sha256};
use validator::{ValidationErrors, ValidationErrorsKind};

#[derive(Deserialize, Validate, Debug, Clone, Default)]
pub struct NewTask {
    #[validate(required, length(min = 1))]
    pub name: Option<String>,

    #[validate(required)]
    pub leader_aggregator_id: Option<String>,

    #[validate(required)]
    pub helper_aggregator_id: Option<String>,

    #[validate(required_nested)]
    pub vdaf: Option<Vdaf>,

    #[validate(required, range(min = 100))]
    pub min_batch_size: Option<u64>,

    #[validate(range(min = 0))]
    pub max_batch_size: Option<u64>,

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

    let Some(aggregator) = Aggregators::find_by_id(id).one(db).await? else {
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
    rand::thread_rng().fill(&mut verify_key);
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
        db: &impl ConnectionTrait,
        errors: &mut ValidationErrors,
    ) -> Option<CollectorCredential> {
        match self.load_collector_credential(account, db).await {
            Some(collector_credential) => Some(collector_credential),
            None => {
                errors.add("collector_credential_id", ValidationError::new("required"));
                None
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
        let Some(vdaf) = self.vdaf.as_ref() else {
            return None;
        };

        let name = vdaf.name();
        let aggregator_vdaf = match vdaf.representation_for_protocol(protocol) {
            Ok(vdaf) => vdaf,
            Err(e) => {
                let errors = errors.errors_mut().entry("vdaf").or_insert_with(|| {
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
                .entry("vdaf")
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
        errors: &mut ValidationErrors,
    ) {
        let name = QueryType::from(self.max_batch_size).name();
        if !leader.query_types.contains(&name) || !helper.query_types.contains(&name) {
            errors.add("max_batch_size", ValidationError::new("not-supported"));
        }
    }

    pub async fn normalize_and_validate(
        &mut self,
        account: Account,
        db: &impl ConnectionTrait,
    ) -> Result<ProvisionableTask, ValidationErrors> {
        let mut errors = Validate::validate(self).err().unwrap_or_default();
        self.validate_min_lte_max(&mut errors);
        let collector_credential = self
            .validate_collector_credential(&account, db, &mut errors)
            .await;
        let aggregators = self.validate_aggregators(&account, db, &mut errors).await;

        let aggregator_vdaf = if let Some((leader, helper, protocol)) = aggregators.as_ref() {
            self.validate_query_type_is_supported(leader, helper, &mut errors);
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
                aggregator_vdaf: aggregator_vdaf.unwrap(),
                min_batch_size: self.min_batch_size.unwrap(),
                max_batch_size: self.max_batch_size,
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
