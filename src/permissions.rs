use crate::{
    entity::Membership,
    handler::{account_bearer_token::AccountBearerToken, Error},
    Db, User,
};
use axum::extract::{FromRef, FromRequestParts};
use axum::http::{self, request::Parts};
use trillium::Conn;
use trillium_api::FromConn;

#[derive(Debug, Clone)]
pub enum PermissionsActor {
    ApiToken(AccountBearerToken),
    User(User, Vec<Membership>),
}

impl PermissionsActor {
    pub fn is_admin(&self) -> bool {
        match self {
            PermissionsActor::ApiToken(token) => token.account.admin,
            PermissionsActor::User(user, _) => user.is_admin(),
        }
    }

    fn check_permission<T: Permissions>(&self, is_safe: bool, t: &T) -> bool {
        if is_safe {
            t.allow_read(self)
        } else {
            t.allow_write(self)
        }
    }

    pub fn is_allowed<T: Permissions>(&self, method: trillium::Method, t: &T) -> bool {
        self.check_permission(method.is_safe(), t)
    }

    pub fn if_allowed<T: Permissions>(&self, method: trillium::Method, t: T) -> Option<T> {
        if self.is_allowed(method, &t) {
            Some(t)
        } else {
            None
        }
    }

    /// Axum-side equivalent of [`is_allowed`](Self::is_allowed).
    pub fn is_allowed_http<T: Permissions>(&self, method: &http::Method, t: &T) -> bool {
        self.check_permission(method.is_safe(), t)
    }

    /// Axum-side equivalent of [`if_allowed`](Self::if_allowed).
    pub fn if_allowed_http<T: Permissions>(&self, method: &http::Method, t: T) -> Option<T> {
        if self.is_allowed_http(method, &t) {
            Some(t)
        } else {
            None
        }
    }

    pub fn is_user(&self) -> bool {
        matches!(self, Self::User(_, _))
    }

    pub fn is_token(&self) -> bool {
        matches!(self, Self::ApiToken(_))
    }

    pub fn account_ids(&self) -> Vec<uuid::Uuid> {
        match self {
            PermissionsActor::ApiToken(token) => vec![token.account.id],
            PermissionsActor::User(_, memberships) => {
                memberships.iter().map(|m| m.account_id).collect()
            }
        }
    }
}

#[trillium::async_trait]
impl FromConn for PermissionsActor {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        if let Some(actor) = conn.state::<Self>() {
            return Some(actor.clone());
        }
        let abt = AccountBearerToken::from_conn(conn).await;
        let user = User::from_conn(conn).await;
        let actor = match (abt, user) {
            (Some(abt), None) => Some(Self::ApiToken(abt)),
            (None, Some(user)) => {
                let db: &Db = conn.state()?;
                let memberships = user.memberships().all(db).await.ok()?;
                Some(Self::User(user, memberships))
            }
            _ => None,
        };

        if let Some(actor) = &actor {
            conn.insert_state(actor.clone());
        }

        actor
    }
}

// ---------------------------------------------------------------------------
// Axum extractor — mirrors the Trillium FromConn above
// ---------------------------------------------------------------------------

impl<S> FromRequestParts<S> for PermissionsActor
where
    Db: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Error> {
        // Cache: return early if already extracted for this request.
        if let Some(actor) = parts.extensions.get::<Self>() {
            return Ok(actor.clone());
        }

        let abt = AccountBearerToken::from_parts(parts, state).await;
        let user = User::from_parts(parts, state).await?;

        // Match the Trillium behavior: if both a bearer token and a session
        // user are present, reject the request. A request should authenticate
        // via exactly one mechanism.
        let actor = match (abt, user) {
            (Some(abt), None) => Self::ApiToken(abt),
            (None, Some(user)) => {
                let db = Db::from_ref(state);
                let memberships = user.memberships().all(&db).await.map_err(Error::from)?;
                Self::User(user, memberships)
            }
            // Both present, or neither present.
            _ => return Err(Error::AccessDenied),
        };

        parts.extensions.insert(actor.clone());
        Ok(actor)
    }
}

pub trait Permissions {
    fn allow_read(&self, actor: &PermissionsActor) -> bool {
        self.allow_write(actor)
    }

    fn allow_write(&self, _actor: &PermissionsActor) -> bool {
        false
    }
}
