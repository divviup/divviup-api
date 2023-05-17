use super::*;

const URL_SAFE_BASE64_CHARS: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn url_safe_base64(data: &str) -> Result<(), ValidationError> {
    if data
        .chars()
        .all(|c| URL_SAFE_BASE64_CHARS.contains(&(c as u8)))
    {
        Ok(())
    } else {
        Err(ValidationError::new("base64"))
    }
}

fn in_the_future(time: &TimeDateTimeWithTimeZone) -> Result<(), ValidationError> {
    if time < &TimeDateTimeWithTimeZone::now_utc() {
        Err(ValidationError::new("past"))
    } else {
        Ok(())
    }
}

#[derive(Deserialize, Validate, Debug, Clone, Default)]
pub struct NewTask {
    #[validate(length(equal = 43), custom = "url_safe_base64")] // 32 bytes after base64 decode
    pub id: Option<String>,

    #[validate(length(equal = 22), custom = "url_safe_base64")] // 16 bytes after base64 decode
    pub vdaf_verify_key: Option<String>,

    #[validate(length(min = 1))]
    pub aggregator_auth_token: Option<String>,

    #[validate(length(min = 1))]
    pub collector_auth_token: Option<String>,

    #[validate(required, length(min = 1))]
    pub name: Option<String>,

    #[validate(required, url)]
    pub partner_url: Option<String>,

    #[validate(required_nested)]
    pub vdaf: Option<Vdaf>,

    #[validate(required, range(min = 100))]
    pub min_batch_size: Option<u64>,

    #[validate(range(min = 0))]
    pub max_batch_size: Option<u64>,

    #[validate(required)]
    pub is_leader: Option<bool>,

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

    #[validate(required_nested)]
    pub hpke_config: Option<HpkeConfig>,
}

#[cfg(test)]
mod tests {
    use std::iter::repeat_with;

    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use rand::random;

    use super::*;
    use crate::test::assert_errors;
    #[test]
    fn validation() {
        assert_errors(
            NewTask {
                id: Some("tooshort".into()),
                ..Default::default()
            },
            "id",
            &["length"],
        );

        assert_errors(
            NewTask {
                id: Some("🦀".into()),
                ..Default::default()
            },
            "id",
            &["length", "base64"],
        );

        assert_errors(
            NewTask {
                id: Some(std::iter::repeat(' ').take(43).collect()),
                ..Default::default()
            },
            "id",
            &["base64"],
        );

        assert_errors(
            NewTask {
                id: Some(
                    URL_SAFE_NO_PAD.encode(repeat_with(random::<u8>).take(32).collect::<Vec<_>>()),
                ),
                ..Default::default()
            },
            "id",
            &[],
        );

        assert_errors(
            NewTask {
                vdaf_verify_key: Some(
                    URL_SAFE_NO_PAD.encode(repeat_with(random::<u8>).take(16).collect::<Vec<_>>()),
                ),
                ..Default::default()
            },
            "vdaf_verify_key",
            &[],
        );
    }
}
