use crate::{CliResult, Error, Output};
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine,
};
use clap::Subcommand;
use divviup_client::{DivviupClient, Uuid};
use janus_messages::codec::Encode;
use serde_json::json;
use std::{borrow::Cow, path::PathBuf};
use trillium_tokio::tokio::fs;

mod generate;
use generate::*;

#[derive(Subcommand, Debug)]
pub enum HpkeConfigAction {
    List,
    Create {
        #[arg(long, short, required_unless_present("base64"))]
        file: Option<PathBuf>,
        #[arg(long, short, required_unless_present("file"))]
        base64: Option<String>,
        name: Option<String>,
    },
    Delete {
        hpke_config_id: Uuid,
    },
    Generate {
        #[arg(long, default_value = "X25519HkdfSha256")]
        kem: Kem,
        #[arg(long, default_value = "HkdfSha256")]
        kdf: Kdf,
        #[arg(long, default_value = "Aes128Gcm")]
        aead: Aead,
        #[arg(long)]
        id: Option<u8>,
        #[arg(long, short)]
        name: Option<String>,
    },
}

impl HpkeConfigAction {
    pub(crate) async fn run(
        self,
        account_id: Uuid,
        client: DivviupClient,
        output: Output,
    ) -> CliResult {
        match self {
            HpkeConfigAction::List => {
                output.display(client.hpke_configs(account_id).await?);
            }

            HpkeConfigAction::Create { file, base64, name } => {
                let bytes = match (&file, base64) {
                    (Some(path), None) => fs::read(path).await?,
                    (None, Some(base64)) => STANDARD.decode(base64)?,
                    (Some(_), Some(_)) => {
                        return Err(Error::Other(
                            "path and base64 are mutually exclusive".into(),
                        ));
                    }
                    (None, None) => unreachable!(),
                };

                let name = match (name, &file) {
                    (Some(name), _) => Some(Cow::Owned(name)),
                    (_, Some(file)) => file.file_name().map(|s| s.to_string_lossy()),
                    _ => None,
                };

                output.display(
                    client
                        .create_hpke_config(account_id, bytes, name.as_deref())
                        .await?,
                );
            }

            HpkeConfigAction::Delete { hpke_config_id } => {
                client.delete_hpke_config(hpke_config_id).await?;
            }

            HpkeConfigAction::Generate {
                kem,
                kdf,
                aead,
                name,
                id,
            } => {
                let hpke_dispatch::Keypair {
                    private_key,
                    public_key,
                } = kem.gen_keypair();

                let config_id = id.unwrap_or_else(|| rand::random());

                let hpke_config = janus_messages::HpkeConfig::new(
                    config_id.into(),
                    (kem.0 as u16).try_into().unwrap(),
                    (kdf.0 as u16).try_into().unwrap(),
                    (aead.0 as u16).try_into().unwrap(),
                    janus_messages::HpkePublicKey::from(public_key.clone()),
                );

                let name = name.unwrap_or_else(|| format!("hpke-config-{config_id}"));
                output.display(
                    client
                        .create_hpke_config(account_id, hpke_config.get_encoded(), Some(&name))
                        .await?,
                );
                output.display(json!({
                    "id": config_id,
                    "public_key": URL_SAFE_NO_PAD.encode(public_key),
                    "private_key": URL_SAFE_NO_PAD.encode(private_key),
                    "kem": kem.0,
                    "kdf": kdf.0,
                    "aead": aead.0
                }));
            }
        }
        Ok(())
    }
}
