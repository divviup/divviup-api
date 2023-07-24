use crate::{CliResult, Output};
use clap::Subcommand;
use divviup_client::{DivviupClient, Uuid};
use email_address::EmailAddress;

#[derive(Subcommand, Debug)]
pub enum MembershipAction {
    List,
    Create { email: EmailAddress },
    Delete { membership_id: Uuid },
}

impl MembershipAction {
    pub(crate) async fn run(
        self,
        account_id: Uuid,
        client: DivviupClient,
        output: Output,
    ) -> CliResult {
        match self {
            MembershipAction::List => output.display(client.memberships(account_id).await?),

            MembershipAction::Create { email } => {
                output.display(client.create_membership(account_id, email.as_ref()).await?)
            }

            MembershipAction::Delete { membership_id } => {
                client.delete_membership(membership_id).await?;
            }
        }
        Ok(())
    }
}
