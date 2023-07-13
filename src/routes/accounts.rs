use crate::{
    entity::{
        Account, Accounts, CreateMembership, MembershipColumn, Memberships, NewAccount,
        UpdateAccount,
    },
    handler::Error,
    user::User,
    Db,
};
use sea_orm::{prelude::*, EntityTrait, TransactionTrait};
use trillium::{async_trait, Conn, Handler, Status};
use trillium_api::{FromConn, Json};
use trillium_caching_headers::CachingHeadersExt;
use trillium_router::RouterConnExt;

#[async_trait]
impl FromConn for Account {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        let db = Db::from_conn(conn).await?;
        let user = User::from_conn(conn).await?;
        let account_id = conn.param("account_id")?;
        let account_id = Uuid::parse_str(account_id).ok()?;

        let account = if user.is_admin() {
            Accounts::find_by_id(account_id).one(&db).await
        } else {
            Accounts::find_by_id(account_id)
                .inner_join(Memberships)
                .filter(MembershipColumn::UserEmail.eq(&user.email))
                .one(&db)
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

pub async fn index(_: &mut Conn, (user, db): (User, Db)) -> Result<impl Handler, Error> {
    let accounts = if user.is_admin() {
        Accounts::find().all(&db).await?
    } else {
        Accounts::find()
            .inner_join(Memberships)
            .filter(MembershipColumn::UserEmail.eq(&user.email))
            .all(&db)
            .await?
    };

    Ok(Json(accounts))
}

pub async fn create(
    _: &mut Conn,
    (Json(new_account), current_user, db): (Json<NewAccount>, User, Db),
) -> Result<impl Handler, Error> {
    let transaction = db.begin().await?;
    let account = new_account.build()?.insert(&transaction).await?;
    let membership = CreateMembership {
        user_email: Some(current_user.email),
    };
    membership.build(&account)?.insert(&transaction).await?;
    transaction.commit().await?;
    Ok((Json(account), Status::Accepted))
}

pub async fn update(
    _: &mut Conn,
    (account, Json(update_account), db): (Account, Json<UpdateAccount>, Db),
) -> Result<impl Handler, Error> {
    let account = update_account.build(account)?.update(&db).await?;
    Ok((Json(account), Status::Accepted))
}
