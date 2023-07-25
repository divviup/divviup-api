use crate::{
    entity::{Account, Accounts, CreateMembership, NewAccount, UpdateAccount},
    handler::Error,
    Db, Permissions, PermissionsActor,
};
use sea_orm::{ActiveModelTrait, EntityTrait, TransactionTrait};
use trillium::{async_trait, Conn, Handler, Status};
use trillium_api::{FromConn, Json};
use trillium_caching_headers::CachingHeadersExt;
use trillium_router::RouterConnExt;
use uuid::Uuid;

impl Permissions for Account {
    fn allow_write(&self, actor: &PermissionsActor) -> bool {
        actor.is_admin() || actor.account_ids().contains(&self.id)
    }
}

#[async_trait]
impl FromConn for Account {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        let actor = PermissionsActor::from_conn(conn).await?;
        let db: &Db = conn.state()?;
        let account_id = conn.param("account_id")?.parse::<Uuid>().ok()?;
        match Accounts::find_by_id(account_id).one(db).await {
            Ok(Some(account)) => actor.if_allowed(conn.method(), account),
            Ok(None) => None,
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

pub async fn index(
    _: &mut Conn,
    (actor, db): (PermissionsActor, Db),
) -> Result<impl Handler, Error> {
    Accounts::for_actor(&actor)
        .all(&db)
        .await
        .map(Json)
        .map_err(Error::from)
}

pub async fn create(
    _: &mut Conn,
    (Json(new_account), actor, db): (Json<NewAccount>, PermissionsActor, Db),
) -> Result<impl Handler, Error> {
    if !(actor.is_user() || actor.is_admin()) {
        return Err(Error::AccessDenied);
    }

    let transaction = db.begin().await?;
    let account = new_account.build()?.insert(&transaction).await?;
    if let PermissionsActor::User(user, _) = actor {
        let membership = CreateMembership {
            user_email: Some(user.email),
        };
        membership.build(&account)?.insert(&transaction).await?;
    }
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
