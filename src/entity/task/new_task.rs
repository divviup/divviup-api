use super::*;
use crate::{
    clients::aggregator_client::api_types::{Decode, HpkeConfig},
    entity::{Account, Aggregator, Aggregators},
    handler::Error,
};
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine,
};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::io::Cursor;
use validator::ValidationErrors;

fn in_the_future(time: &TimeDateTimeWithTimeZone) -> Result<(), ValidationError> {
    if time < &TimeDateTimeWithTimeZone::now_utc() {
        Err(ValidationError::new("past"))
    } else {
        Ok(())
    }
}

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

    #[validate(custom = "in_the_future")]
    #[serde(default, with = "time::serde::iso8601::option")]
    pub expiration: Option<TimeDateTimeWithTimeZone>,

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
    pub hpke_config: Option<String>,
}

fn hpke_config(base64: &str) -> Result<HpkeConfig, ValidationError> {
    if base64.is_empty() {
        return Err(ValidationError::new("required"));
    }
    let bytes = STANDARD
        .decode(base64)
        .map_err(|_| ValidationError::new("base64"))?;
    let mut cursor = Cursor::new(bytes.as_slice());
    let hpke_config = HpkeConfig::decode(&mut cursor).map_err(|e| ValidationError {
        code: "hpke_config".into(),
        message: Some(e.to_string().into()),
        params: Default::default(),
    })?;

    Ok(hpke_config)
}

async fn load_aggregator(
    account: &Account,
    id: Option<&str>,
    db: &impl ConnectionTrait,
) -> Result<Option<Aggregator>, Error> {
    let Some(id) = id.map(Uuid::parse_str).transpose()? else { return Ok(None) };
    let Some(aggregator) = Aggregators::find_by_id(id).one(db).await? else { return Ok(None) };

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
        URL_SAFE_NO_PAD.encode(Sha256::digest(&verify_key)),
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

    fn validate_hpke_config(&self, errors: &mut ValidationErrors) -> Option<HpkeConfig> {
        match hpke_config(self.hpke_config.as_ref()?) {
            Ok(hpke_config) => Some(hpke_config),
            Err(e) => {
                errors.add("hpke_config", e);
                None
            }
        }
    }

    async fn validate_aggregators(
        &self,
        account: &Account,
        db: &impl ConnectionTrait,
        errors: &mut ValidationErrors,
    ) -> Option<(Aggregator, Aggregator)> {
        let leader = load_aggregator(account, self.leader_aggregator_id.as_deref(), db)
            .await
            .ok()
            .flatten();
        if leader.is_none() {
            errors.add("leader_aggregator_id", ValidationError::new("missing"));
        }

        let helper = load_aggregator(account, self.helper_aggregator_id.as_deref(), db)
            .await
            .ok()
            .flatten();
        if helper.is_none() {
            errors.add("helper_aggregator_id", ValidationError::new("missing"));
        }

        let (Some(leader), Some(helper)) = (leader, helper) else { return None };

        if leader == helper {
            errors.add("leader_aggregator_id", ValidationError::new("same"));
            errors.add("helper_aggregator_id", ValidationError::new("same"));
        }

        if !leader.is_first_party() && !helper.is_first_party() {
            errors.add(
                "leader_aggregator_id",
                ValidationError::new("no-first-party"),
            );
            errors.add(
                "helper_aggregator_id",
                ValidationError::new("no-first-party"),
            );
        }

        if errors.is_empty() {
            Some((leader, helper))
        } else {
            None
        }
    }

    pub async fn validate(
        &self,
        account: Account,
        db: &impl ConnectionTrait,
    ) -> Result<ProvisionableTask, ValidationErrors> {
        let mut errors = Validate::validate(self).err().unwrap_or_default();
        self.validate_min_lte_max(&mut errors);
        let hpke_config = self.validate_hpke_config(&mut errors);
        let aggregators = self.validate_aggregators(&account, db, &mut errors).await;

        if errors.is_empty() {
            // Unwrap safety: All of these unwraps below have previously
            // been checked by the above validations. The fact that we
            // have to check them twice is a consequence of the
            // disharmonious combination of Validate and the fact that we
            // need to use options for all fields so serde doesn't bail on
            // the first error.
            let (leader_aggregator, helper_aggregator) = aggregators.unwrap();

            let (vdaf_verify_key, id) = generate_vdaf_verify_key_and_expected_task_id();

            Ok(ProvisionableTask {
                account,
                id,
                vdaf_verify_key,
                name: self.name.clone().unwrap(),
                leader_aggregator,
                helper_aggregator,
                vdaf: self.vdaf.clone().unwrap(),
                min_batch_size: self.min_batch_size.unwrap(),
                max_batch_size: self.max_batch_size,
                expiration: self.expiration,
                time_precision_seconds: self.time_precision_seconds.unwrap(),
                hpke_config: hpke_config.unwrap(),
            })
        } else {
            Err(errors)
        }
    }
}
