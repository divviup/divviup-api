pub mod account;
pub mod membership;
pub mod queue;
pub mod session;
pub mod task;

#[macro_use]
pub mod macros;

pub use account::{
    Column as AccountColumn, Entity as Accounts, Model as Account, NewAccount, UpdateAccount,
};
pub use membership::{
    Column as MembershipColumn, CreateMembership, Entity as Memberships, Model as Membership,
};
pub use task::{Column as TaskColumn, Entity as Tasks, Model as Task, NewTask, UpdateTask};

pub use session::{Column as SessionColumn, Entity as Sessions, Model as Session};

const URL_SAFE_BASE64_CHARS: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn url_safe_base64(data: &str) -> Result<(), validator::ValidationError> {
    if data
        .chars()
        .all(|c| u8::try_from(c).map_or(false, |c| URL_SAFE_BASE64_CHARS.contains(&c)))
    {
        Ok(())
    } else {
        Err(validator::ValidationError::new("base64"))
    }
}
