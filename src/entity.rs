pub mod account;
pub mod aggregator;
pub mod api_token;
pub mod membership;
pub mod queue;
pub mod session;
pub mod task;
mod url;

#[macro_use]
pub mod macros;

pub use account::{
    Column as AccountColumn, Entity as Accounts, Model as Account, NewAccount,
    Relation as AccountRelation, UpdateAccount,
};
pub use membership::{
    Column as MembershipColumn, CreateMembership, Entity as Memberships, Model as Membership,
};
pub use task::{
    Column as TaskColumn, Entity as Tasks, Model as Task, NewTask, ProvisionableTask, UpdateTask,
};

pub use session::{Column as SessionColumn, Entity as Sessions, Model as Session};

pub use aggregator::{
    Column as AggregatorColumn, Entity as Aggregators, Model as Aggregator, NewAggregator,
    UpdateAggregator,
};

pub use api_token::{Column as ApiTokenColumn, Entity as ApiTokens, Model as ApiToken};

mod validators {
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub(super) fn base64(data: &str) -> Result<(), validator::ValidationError> {
        if data
            .chars()
            .all(|c| u8::try_from(c).map_or(false, |c| BASE64_CHARS.contains(&c)))
        {
            Ok(())
        } else {
            Err(validator::ValidationError::new("base64"))
        }
    }
}
