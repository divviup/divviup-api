use crate::{CliResult, DetermineAccountId, Error, Output};
use base64::{engine::general_purpose::STANDARD, Engine};
use clap::Subcommand;
use divviup_client::{Decode, DivviupClient, HpkeConfigContents, Uuid};
use std::{borrow::Cow, path::PathBuf};
use trillium_tokio::tokio::fs;

#[cfg(feature = "hpke")]
use hpke_dispatch::{Aead, Kdf, Kem};

#[derive(Subcommand, Debug)]
pub enum HpkeConfigAction {
    /// list hpke configs for the target account
    List,
    /// list hpke configs for the target account
    Create {
        #[arg(
            long,
            short,
            required_unless_present("base64"),
            conflicts_with("base64")
        )]
        /// filesystem path to a dap-encoded hpke config file
        file: Option<PathBuf>,

        #[arg(long, short, required_unless_present("file"), conflicts_with("file"))]
        /// standard-base64 dap-encoded hpke config
        base64: Option<String>,

        #[arg(short, long)]
        /// optional display name for this hpke config
        ///
        /// if `file` is provided and `name` is not, the filename will be used
        name: Option<String>,
    },
    /// delete a hpke config by id
    Delete { hpke_config_id: Uuid },

    #[cfg(feature = "hpke")]
    /// create a new hpke config and upload the public key to divviup
    ///
    /// the private key will be output to stdout
    /// but not recorded anywhere else
    Generate {
        #[arg(short, long, default_value = "x25519-sha256")]
        /// key encapsulation mechanism
        kem: Kem,

        /// key derivation function
        #[arg(long, default_value = "sha256")]
        kdf: Kdf,

        /// authenticated encryption with additional data
        #[arg(long, default_value = "aes128-gcm")]
        aead: Aead,

        /// an optional u8 identifier to distinguish from other hpke configs in the dap protocol
        ///
        /// note that this is distinct from the uuid used to represent this hpke config in the
        /// divviup api
        #[arg(long)]
        id: Option<u8>,

        /// an optional display name to identify this hpke config
        #[arg(long, short)]
        name: Option<String>,
    },
}

impl HpkeConfigAction {
    pub(crate) async fn run(
        self,
        account_id: DetermineAccountId,
        client: DivviupClient,
        output: Output,
    ) -> CliResult {
        match self {
            HpkeConfigAction::List => {
                let account_id = account_id.await?;
                output.display(client.hpke_configs(account_id).await?);
            }

            HpkeConfigAction::Create { file, base64, name } => {
                let account_id = account_id.await?;
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

                let hpke_config = HpkeConfigContents::get_decoded(&bytes)?;

                output.display(
                    client
                        .create_hpke_config(account_id, &hpke_config, name.as_deref())
                        .await?,
                );
            }

            HpkeConfigAction::Delete { hpke_config_id } => {
                client.delete_hpke_config(hpke_config_id).await?;
            }

            #[cfg(feature = "hpke")]
            HpkeConfigAction::Generate {
                kem,
                kdf,
                aead,
                name,
                id,
            } => {
                use base64::engine::general_purpose::URL_SAFE_NO_PAD;
                use serde_json::json;
                let account_id = account_id.await?;

                let hpke_dispatch::Keypair {
                    private_key,
                    public_key,
                } = kem.gen_keypair();

                let config_id = id.unwrap_or_else(|| rand::random());

                let hpke_config = HpkeConfigContents::new(
                    config_id.into(),
                    (kem as u16).try_into().unwrap(),
                    (kdf as u16).try_into().unwrap(),
                    (aead as u16).try_into().unwrap(),
                    public_key.clone().into(),
                );

                let name = name.unwrap_or_else(|| format!("hpke-config-{config_id}"));
                output.display(
                    client
                        .create_hpke_config(account_id, &hpke_config, Some(&name))
                        .await?,
                );
                output.display(json!({
                    "id": config_id,
                    "public_key": URL_SAFE_NO_PAD.encode(public_key),
                    "private_key": URL_SAFE_NO_PAD.encode(private_key),
                    "kem": kem,
                    "kdf": kdf,
                    "aead": aead
                }));
            }
        }
        Ok(())
    }
}
