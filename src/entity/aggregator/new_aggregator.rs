use super::ActiveModel;
use crate::{
    clients::{AggregatorClient, ClientError},
    entity::{url::Url, Account, Aggregator},
    handler::Error,
};
use sea_orm::IntoActiveModel;
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    str::FromStr,
};
use time::OffsetDateTime;
use tokio::net::lookup_host;
use trillium_client::Client;
use trillium_http::Status;
use url::Host;
use uuid::Uuid;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Deserialize, Serialize, Validate, Debug, Clone, Default)]
pub struct NewAggregator {
    #[validate(required, length(min = 1, max = 255))]
    pub name: Option<String>,
    #[cfg_attr(
        not(feature = "integration-testing"),
        validate(custom(function = "https"))
    )]
    #[validate(length(max = 2048))]
    pub api_url: Option<String>,
    #[validate(length(max = 4096))]
    pub bearer_token: Option<String>,
    pub is_first_party: Option<bool>,
}

#[cfg_attr(feature = "integration-testing", allow(dead_code))]
fn https(url: &str) -> Result<(), ValidationError> {
    let url = url::Url::from_str(url).map_err(|_| ValidationError::new("https-url"))?;
    if url.scheme() != "https" {
        return Err(ValidationError::new("https-url"));
    }
    Ok(())
}

async fn validate_aggregator_url(url: &url::Url) -> Result<(), Error> {
    fn ssrf_error(code: &'static str) -> Error {
        let mut ve = ValidationErrors::new();
        ve.add("api_url", ValidationError::new(code));
        ve.into()
    }

    let host = url.host_str().ok_or_else(|| ssrf_error("invalid-url"))?;
    let port = url.port().unwrap_or(443);

    // For IP-literal hosts, check directly without DNS.
    match url.host() {
        Some(Host::Ipv4(ip)) => {
            if is_private_ipv4(ip) {
                return Err(ssrf_error("private-address"));
            }
            return Ok(());
        }
        Some(Host::Ipv6(ip)) => {
            if is_private_ipv6(ip) {
                return Err(ssrf_error("private-address"));
            }
            return Ok(());
        }
        _ => {}
    }

    // Reject "localhost" before resolving.
    if host.eq_ignore_ascii_case("localhost") {
        return Err(ssrf_error("private-address"));
    }

    // Resolve the hostname and check all resulting addresses.
    let addrs = lookup_host(format!("{host}:{port}"))
        .await
        .map_err(|_| ssrf_error("dns-resolution-failed"))?;

    for addr in addrs {
        if is_private_ip(addr.ip()) {
            return Err(ssrf_error("private-address"));
        }
    }

    Ok(())
}

fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_private_ipv4(ip),
        IpAddr::V6(ip) => is_private_ipv6(ip),
    }
}

fn is_private_ipv4(ip: Ipv4Addr) -> bool {
    ip.is_loopback()         // 127.0.0.0/8
        || ip.is_private()   // 10/8, 172.16/12, 192.168/16
        || ip.is_link_local()   // 169.254.0.0/16 (includes cloud metadata)
        || ip.is_broadcast()    // 255.255.255.255
        || ip.is_unspecified()  // 0.0.0.0
        || ip.is_documentation() // 192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24
        || ip.octets()[0] == 100 && (ip.octets()[1] & 0xC0) == 64 // 100.64.0.0/10 (RFC 6598)
}

fn is_private_ipv6(ip: Ipv6Addr) -> bool {
    ip.is_loopback()         // ::1
        || ip.is_unspecified() // ::
        || {
            if let Some(ipv4) = ip.to_ipv4_mapped() {
                is_private_ipv4(ipv4)
            } else {
                // Unique local (fc00::/7) and link-local (fe80::/10)
                let segments = ip.segments();
                (segments[0] & 0xfe00) == 0xfc00 || (segments[0] & 0xffc0) == 0xfe80
            }
        }
}

impl NewAggregator {
    pub async fn build(
        self,
        account: Option<&Account>,
        client: Client,
        crypter: &crate::Crypter,
        ssrf_validation_enabled: bool,
    ) -> Result<ActiveModel, Error> {
        self.validate()?;

        let api_url: Url = self.api_url.as_ref().unwrap().parse()?;

        if ssrf_validation_enabled {
            validate_aggregator_url(&api_url).await?;
        }

        let aggregator_config = AggregatorClient::get_config(
            client,
            api_url.clone().into(),
            self.bearer_token.as_ref().unwrap(),
        )
        .await
        .map_err(|e| match e {
            ClientError::HttpStatusNotSuccess {
                status: Some(Status::Unauthorized | Status::Forbidden),
                ..
            } => {
                let mut ve = ValidationErrors::new();
                ve.add("bearer_token", ValidationError::new("token-not-recognized"));
                ve.into()
            }

            ClientError::Http(_)
            | ClientError::HttpStatusNotSuccess {
                status: Some(Status::NotFound),
                ..
            } => {
                let mut ve = ValidationErrors::new();
                ve.add("api_url", ValidationError::new("http-error"));
                ve.into()
            }

            other => Error::from(other),
        })?;

        // unwrap safety: the below unwraps will never panic, because
        // the above call to `NewAggregator::validate` will
        // early-return if any of the required `Option`s is `None`.
        //
        // This is an unfortunate consequence of the combination of
        // `serde` and `validate`, and would be resolved by a
        // potential deserializer-and-validator library that
        // accumulates errors instead of bailing on the first
        // error. As this deserialize-and-validate behavior is outside
        // of the scope of this repository, we work around this by
        // double-checking these Options -- once in validate, and
        // again in the conversion to non-optional fields.

        let encrypted_bearer_token = crypter.encrypt(
            api_url.as_ref().as_bytes(),
            self.bearer_token.as_deref().unwrap_or_default().as_bytes(),
        )?;

        Ok(Aggregator {
            role: aggregator_config.role,
            name: self.name.unwrap(),
            api_url,
            dap_url: aggregator_config.dap_url.into(),
            encrypted_bearer_token,
            id: Uuid::new_v4(),
            account_id: account.map(|account| account.id),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            deleted_at: None,
            is_first_party: account.is_none() && self.is_first_party.unwrap_or(true),
            query_types: aggregator_config.query_types.into(),
            vdafs: aggregator_config.vdafs.into(),
            protocol: aggregator_config.protocol,
            features: aggregator_config.features.into(),
        }
        .into_active_model())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_ipv4_addresses() {
        assert!(is_private_ipv4(Ipv4Addr::new(127, 0, 0, 1))); // loopback
        assert!(is_private_ipv4(Ipv4Addr::new(10, 0, 0, 1))); // RFC 1918
        assert!(is_private_ipv4(Ipv4Addr::new(172, 16, 0, 1))); // RFC 1918
        assert!(is_private_ipv4(Ipv4Addr::new(192, 168, 1, 1))); // RFC 1918
        assert!(is_private_ipv4(Ipv4Addr::new(169, 254, 169, 254))); // link-local / metadata
        assert!(is_private_ipv4(Ipv4Addr::new(0, 0, 0, 0))); // unspecified
        assert!(is_private_ipv4(Ipv4Addr::new(100, 64, 0, 1))); // RFC 6598
        assert!(is_private_ipv4(Ipv4Addr::new(100, 127, 255, 255))); // RFC 6598 upper
        assert!(!is_private_ipv4(Ipv4Addr::new(8, 8, 8, 8))); // public
        assert!(!is_private_ipv4(Ipv4Addr::new(100, 128, 0, 1))); // just outside RFC 6598
    }

    #[test]
    fn private_ipv6_addresses() {
        assert!(is_private_ipv6(Ipv6Addr::LOCALHOST)); // ::1
        assert!(is_private_ipv6(Ipv6Addr::UNSPECIFIED)); // ::
        assert!(is_private_ipv6("fc00::1".parse().unwrap())); // unique local
        assert!(is_private_ipv6("fe80::1".parse().unwrap())); // link-local
        assert!(is_private_ipv6("::ffff:127.0.0.1".parse().unwrap())); // IPv4-mapped loopback
        assert!(is_private_ipv6("::ffff:169.254.169.254".parse().unwrap())); // IPv4-mapped metadata
        assert!(!is_private_ipv6("2001:4860:4860::8888".parse().unwrap())); // public
    }

    #[test]
    fn private_ip_dispatch() {
        assert!(is_private_ip(IpAddr::V4(Ipv4Addr::LOCALHOST)));
        assert!(is_private_ip(IpAddr::V6(Ipv6Addr::LOCALHOST)));
        assert!(!is_private_ip(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
    }

    fn url(s: &str) -> url::Url {
        url::Url::parse(s).unwrap()
    }

    #[tokio::test]
    async fn validate_rejects_private_ip_literals() {
        assert!(
            validate_aggregator_url(&url("https://169.254.169.254/latest/"))
                .await
                .is_err()
        );
        assert!(validate_aggregator_url(&url("https://127.0.0.1/"))
            .await
            .is_err());
        assert!(validate_aggregator_url(&url("https://10.0.0.1/"))
            .await
            .is_err());
        assert!(validate_aggregator_url(&url("https://[::1]/"))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn validate_rejects_localhost_hostname() {
        assert!(validate_aggregator_url(&url("https://localhost/"))
            .await
            .is_err());
        assert!(validate_aggregator_url(&url("https://LOCALHOST/"))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn validate_accepts_public_ip_literal() {
        assert!(validate_aggregator_url(&url("https://8.8.8.8/"))
            .await
            .is_ok());
    }
}
