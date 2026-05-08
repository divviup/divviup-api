use crate::{
    entity::{Account, Accounts, CreateMembership, NewAccount, UpdateAccount},
    handler::{extract::extract_entity, extract::Json, Error},
    Db, Permissions, PermissionsActor,
};
use axum::{
    extract::{FromRef, FromRequestParts, State},
    http::{header, request::Parts, StatusCode},
    response::IntoResponse,
};
use httpdate::fmt_http_date;
use sea_orm::{ActiveModelTrait, TransactionTrait};

impl Permissions for Account {
    fn allow_write(&self, actor: &PermissionsActor) -> bool {
        actor.is_admin() || actor.account_ids().contains(&self.id)
    }
}

impl<S> FromRequestParts<S> for Account
where
    Db: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Error> {
        extract_entity::<Accounts, S>(parts, state, "account_id").await
    }
}

pub async fn show(account: Account) -> impl IntoResponse {
    let last_modified = fmt_http_date(account.updated_at.into());
    ([(header::LAST_MODIFIED, last_modified)], Json(account))
}

pub async fn index(
    actor: PermissionsActor,
    State(db): State<Db>,
) -> Result<Json<Vec<Account>>, Error> {
    Accounts::for_actor(&actor)
        .all(&db)
        .await
        .map(Json)
        .map_err(Error::from)
}

pub async fn create(
    actor: PermissionsActor,
    State(db): State<Db>,
    Json(new_account): Json<NewAccount>,
) -> Result<impl IntoResponse, Error> {
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
    Ok((StatusCode::ACCEPTED, Json(account)))
}

pub async fn update(
    account: Account,
    State(db): State<Db>,
    Json(update_account): Json<UpdateAccount>,
) -> Result<impl IntoResponse, Error> {
    let account = update_account.build(account)?.update(&db).await?;
    Ok((StatusCode::ACCEPTED, Json(account)))
}
