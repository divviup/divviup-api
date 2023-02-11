use crate::handler::oauth2::{JwkFetchError, OauthClient};
use jsonwebtoken::{
    decode, decode_header, jwk::AlgorithmParameters, DecodingKey, Header, Validation,
};
use serde_json::Value;
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Token {
    claims: HashMap<String, Value>,
    header: Header,
    raw: String,
}

#[derive(thiserror::Error, Debug)]
pub enum TokenVerificationError {
    #[error(transparent)]
    JwkError(#[from] jsonwebtoken::errors::Error),

    #[error(transparent)]
    JwkFetchError(#[from] JwkFetchError),

    #[error("Token doesn't have a `kid` header field")]
    MissingKidHeaderField,

    #[error("No matching JWK found for the given kid")]
    NoMatchingJWK,

    #[error("Expected an RSA-signed JWK")]
    ExpectedRSA,
}

use TokenVerificationError::{MissingKidHeaderField, NoMatchingJWK};

impl Token {
    pub async fn verify(token: &str, client: &OauthClient) -> Result<Self, TokenVerificationError> {
        let jwks = client.jwks().await?;
        let header = dbg!(decode_header(token)?);

        let kid = dbg!(header.kid.ok_or(MissingKidHeaderField)?);
        let j = jwks.find(&kid).ok_or(NoMatchingJWK)?;
        let AlgorithmParameters::RSA(rsa) = &j.algorithm else {
            return Err(TokenVerificationError::ExpectedRSA)
        };
        let decoding_key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e)?;
        let validation = Validation::new(j.common.algorithm.unwrap());
        let decoded_token = decode::<HashMap<String, Value>>(token, &decoding_key, &validation)?;

        return Ok(Self {
            claims: decoded_token.claims,
            header: decoded_token.header,
            raw: token.to_string(),
        });
    }
}
