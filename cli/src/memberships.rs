use crate::{CliResult, DetermineAccountId, Output};
use clap::Subcommand;
use divviup_client::{DivviupClient, Uuid};
use email_address::EmailAddress;

#[derive(Subcommand, Debug)]
pub enum MembershipAction {
    /// List all memberships for the target account
    List,

    /// Invite a user by email to the target account
    Create { email: EmailAddress },

    /// Remove a membership by id from the target account
    Delete { membership_id: Uuid },

    /// Remove a membership by email address from the target account
    Remove { email: EmailAddress },
}

impl MembershipAction {
    pub(crate) async fn run(
        self,
        account_id: DetermineAccountId,
        client: DivviupClient,
        output: Output,
    ) -> CliResult {
        match self {
            MembershipAction::List => output.display(client.memberships(account_id.await?).await?),

            MembershipAction::Create { email } => output.display(
                client
                    .create_membership(account_id.await?, email.as_ref())
                    .await?,
            ),

            MembershipAction::Delete { membership_id } => {
                client.delete_membership(membership_id).await?;
            }

            MembershipAction::Remove { email } => {
                if let Some(membership_id) = client
                    .memberships(account_id.await?)
                    .await?
                    .into_iter()
                    .find_map(|membership| {
                        if membership.user_email == email {
                            Some(membership.id)
                        } else {
                            None
                        }
                    })
                {
                    client.delete_membership(membership_id).await?;
                }
            }
        }

        Ok(())
    }
}
