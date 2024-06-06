use crate::{CliResult, DetermineAccountId, Error, Output};
use base64::{engine::general_purpose::STANDARD, Engine};
use clap::Subcommand;
use divviup_client::{Decode, DivviupClient, HpkeConfig, Uuid};
use std::{borrow::Cow, path::PathBuf};
use trillium_tokio::tokio::fs;

#[cfg(feature = "hpke")]
use hpke_dispatch::{Aead, Kdf, Kem};
#[cfg(feature = "hpke")]
use std::{env::current_dir, fs::File, io::Write};

#[derive(Subcommand, Debug)]
pub enum CollectorCredentialAction {
    /// list collector credentials for the target account
    List,
    /// create a new collector credential using the public key from a dap-encoded hpke config file
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
    /// delete a collector credential by uuid
    Delete { collector_credential_id: Uuid },

    #[cfg(feature = "hpke")]
    /// generate a new collector credential and upload the public key to divviup
    ///
    /// the private key will be output to stdout or a local file, but will not be recorded anywhere
    /// else
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

        /// save the generated credential to a file in the current directory
        ///
        /// if `name` is not provided, the filename will be `collector-credential-{id}.json`
        /// where `id` is the id of the newly generated credential.
        ///
        /// if `name` is provided, the filename will be `file.json`
        #[arg(long, short, action)]
        save: bool,
    },
}

impl CollectorCredentialAction {
    pub(crate) async fn run(
        self,
        account_id: DetermineAccountId,
        client: DivviupClient,
        output: Output,
    ) -> CliResult {
        match self {
            CollectorCredentialAction::List => {
                let account_id = account_id.await?;
                output.display(client.collector_credentials(account_id).await?);
            }

            CollectorCredentialAction::Create { file, base64, name } => {
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

                let hpke_config = HpkeConfig::get_decoded(&bytes)?;

                output.display(
                    client
                        .create_collector_credential(account_id, &hpke_config, name.as_deref())
                        .await?,
                );
            }

            CollectorCredentialAction::Delete {
                collector_credential_id,
            } => {
                client
                    .delete_collector_credential(collector_credential_id)
                    .await?;
            }

            #[cfg(feature = "hpke")]
            CollectorCredentialAction::Generate {
                kem,
                kdf,
                aead,
                name,
                id,
                save,
            } => {
                use base64::engine::general_purpose::URL_SAFE_NO_PAD;
                use serde_json::json;
                let account_id = account_id.await?;

                let hpke_dispatch::Keypair {
                    private_key,
                    public_key,
                } = kem.gen_keypair();

                let config_id = id.unwrap_or_else(rand::random);

                let hpke_config = HpkeConfig::new(
                    config_id.into(),
                    (kem as u16).into(),
                    (kdf as u16).into(),
                    (aead as u16).into(),
                    public_key.clone().into(),
                );

                let name = name.unwrap_or_else(|| format!("collector-credential-{config_id}"));
                let collector_credential = client
                    .create_collector_credential(account_id, &hpke_config, Some(&name))
                    .await?;
                let token = collector_credential.token.as_ref().cloned().unwrap();
                let credential = json!({
                    "id": config_id,
                    "public_key": URL_SAFE_NO_PAD.encode(public_key),
                    "private_key": URL_SAFE_NO_PAD.encode(private_key),
                    "kem": kem,
                    "kdf": kdf,
                    "aead": aead,
                    "token": token
                });

                output.display(collector_credential);

                // The collector credential is always written to file in JSON encoding, regardless
                // of the output settings of this CLI. The credential file should be treated as
                // opaque, so we don't grant user control over its encoding.
                if save {
                    let path = current_dir()?.join(name).with_extension("json");
                    let mut file = File::create(path.clone())?;
                    file.write_all(&serde_json::to_vec_pretty(&credential).unwrap())?;
                    println!(
                        "\nSaved new collector credential to {}. Keep this file safe!",
                        path.display()
                    );
                } else {
                    println!(
                        "\nNew collector credential generated. Copy and paste the following text \
                        into a file or your password manager:",
                    );
                    println!("{}", serde_json::to_string_pretty(&credential).unwrap());
                }
            }
        }
        Ok(())
    }
}
