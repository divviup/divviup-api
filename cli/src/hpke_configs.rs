use crate::{CliResult, Error, Output};
use base64::{engine::general_purpose::STANDARD, Engine};
use clap::Subcommand;
use divviup_client::{DivviupClient, Uuid};
use std::{borrow::Cow, path::PathBuf};
use trillium_tokio::tokio::fs;

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
        }
        Ok(())
    }
}
