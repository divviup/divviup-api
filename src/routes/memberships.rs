use crate::{
    entity::{Account, CreateMembership, Membership, MembershipColumn, Memberships},
    queue::Job,
    Db, Error, PermissionsActor,
};
use sea_orm::{
    sea_query::all, ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter,
    TransactionTrait,
};
use trillium::{Conn, Handler, Status};
use trillium_api::Json;

use trillium_router::RouterConnExt;
use uuid::Uuid;

pub async fn index(_: &mut Conn, (account, db): (Account, Db)) -> Result<impl Handler, Error> {
    account
        .find_related(Memberships)
        .all(&db)
        .await
        .map_err(Error::from)
        .map(Json)
}

pub async fn create(
    _: &mut Conn,
    (account, Json(membership), db): (Account, Json<CreateMembership>, Db),
) -> Result<(Json<Membership>, Status), Error> {
    let membership = membership.build(&account)?;
    if let Some(membership) = Memberships::find()
        .filter(all![
            MembershipColumn::AccountId.eq(*membership.account_id.as_ref()),
            MembershipColumn::UserEmail.eq(membership.user_email.as_ref())
        ])
        .one(&db)
        .await?
    {
        return Ok((Json(membership), Status::Ok));
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

    Ok((Json(membership), Status::Created))
}

pub async fn delete(
    conn: &mut Conn,
    (actor, db): (PermissionsActor, Db),
) -> Result<impl Handler, Error> {
    let membership_id = conn
        .param("membership_id")
        .unwrap()
        .parse::<Uuid>()
        .map_err(|_| Error::NotFound)?;

    let membership = Memberships::find_by_id(membership_id)
        .one(&db)
        .await?
        .ok_or(Error::NotFound)?;

    if matches!(&actor, PermissionsActor::User(user, _) if membership.user_email == user.email) {
        Err(Error::AccessDenied)
    } else if actor.is_admin() || actor.account_ids().contains(&membership.account_id) {
        membership.delete(&db).await?;
        Ok(Status::NoContent)
    } else {
        Err(Error::NotFound)
    }
}
