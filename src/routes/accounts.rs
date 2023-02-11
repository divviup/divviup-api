use crate::{entity::*, handler::Error, user::User, DbConnExt};
use sea_orm::{prelude::*, EntityTrait, TransactionTrait};
use trillium::{async_trait, Conn, Handler, Status};
use trillium_api::{FromConn, Json};
use trillium_caching_headers::CachingHeadersExt;
use trillium_router::RouterConnExt;

#[async_trait]
impl FromConn for Account {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        let user = User::from_conn(conn).await?;
        let account_id = conn.param("account_id")?;
        let account_id = Uuid::parse_str(account_id).ok()?;

        let account = if user.is_admin() {
            Accounts::find_by_id(account_id).one(conn.db()).await
        } else {
            Accounts::find_by_id(account_id)
                .inner_join(Memberships)
                .filter(MembershipColumn::UserEmail.eq(&user.email))
                .one(conn.db())
                .await
        };

        match account {
            Ok(account) => account,
            Err(error) => {
                conn.set_state(Error::from(error));
                None
            }
        }
    }
}

pub async fn show(conn: &mut Conn, account: Account) -> Json<Account> {
    conn.set_last_modified(account.updated_at.into());
    Json(account)
}

pub async fn index(conn: &mut Conn, user: User) -> Result<impl Handler, Error> {
    let accounts = if user.is_admin() {
        Accounts::find().all(conn.db()).await?
    } else {
        Accounts::find()
            .inner_join(Memberships)
            .filter(MembershipColumn::UserEmail.eq(&user.email))
            .all(conn.db())
            .await?
    };

    if let Some(last_modified) = accounts.iter().map(|account| account.updated_at).max() {
        conn.set_last_modified(last_modified.into());
    }

    Ok(Json(accounts))
}

pub async fn create(
    conn: &mut Conn,
    (Json(new_account), current_user): (Json<NewAccount>, User),
) -> Result<impl Handler, Error> {
    let transaction = conn.db().begin().await?;
    let account = new_account.build()?.insert(&transaction).await?;
    let membership = CreateMembership {
        user_email: Some(current_user.email),
    };
    membership.build(&account)?.insert(&transaction).await?;
    transaction.commit().await?;
    Ok((Json(account), Status::Accepted))
}

pub async fn update(
    conn: &mut Conn,
    (account, Json(update_account)): (Account, Json<UpdateAccount>),
) -> Result<impl Handler, Error> {
    let account = update_account.build(account)?.update(conn.db()).await?;
    Ok((Json(account), Status::Accepted))
}
