use crate::{
    entity::{Account, CreateMembership, Membership, MembershipColumn, Memberships},
    handler::extract::Json,
    queue::Job,
    Db, Error, PermissionsActor,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::collections::HashMap;
use sea_orm::{
    sea_query::all, ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter,
    TransactionTrait,
};
use uuid::Uuid;

pub async fn index(account: Account, State(db): State<Db>) -> Result<Json<Vec<Membership>>, Error> {
    account
        .find_related(Memberships)
        .all(&db)
        .await
        .map(Json)
        .map_err(Error::from)
}

pub async fn create(
    account: Account,
    State(db): State<Db>,
    Json(membership): Json<CreateMembership>,
) -> Result<impl IntoResponse, Error> {
    let membership = membership.build(&account)?;
    if let Some(membership) = Memberships::find()
        .filter(all![
            MembershipColumn::AccountId.eq(*membership.account_id.as_ref()),
            MembershipColumn::UserEmail.eq(membership.user_email.as_ref())
        ])
        .one(&db)
        .await?
    {
        return Ok((StatusCode::OK, Json(membership)));
    }

    let tx = db.begin().await?;

    let first_membership_for_this_email = Memberships::find()
        .filter(MembershipColumn::UserEmail.eq(membership.user_email.as_ref()))
        .one(&tx)
        .await?
        .is_none();

    let membership = membership.insert(&tx).await?;

    if first_membership_for_this_email && !cfg!(feature = "integration-testing") {
        Job::new_invitation_flow(&membership).insert(&tx).await?;
    }

    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(membership)))
}

pub async fn delete(
    Path(params): Path<HashMap<String, String>>,
    actor: PermissionsActor,
    State(db): State<Db>,
) -> Result<StatusCode, Error> {
    let membership_id = params
        .get("membership_id")
        .and_then(|s| s.parse::<Uuid>().ok())
        .ok_or(Error::NotFound)?;

    let membership = Memberships::find_by_id(membership_id)
        .one(&db)
        .await?
        .ok_or(Error::NotFound)?;

    if matches!(&actor, PermissionsActor::User(user, _) if membership.user_email == user.email) {
        Err(Error::AccessDenied)
    } else if actor.is_admin() || actor.account_ids().contains(&membership.account_id) {
        membership.delete(&db).await?;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(Error::NotFound)
    }
}
