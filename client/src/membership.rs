use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Membership {
    pub id: Uuid,
    pub user_email: EmailAddress,
    pub account_id: Uuid,
}
