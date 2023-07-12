use crate::{entity::Membership, handler::account_bearer_token::AccountBearerToken, Db, User};
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

    pub fn is_allowed<T: Permissions>(&self, method: trillium::Method, t: &T) -> bool {
        if method.is_safe() {
            t.allow_read(self)
        } else {
            t.allow_write(self)
        }
    }

    pub fn if_allowed<T: Permissions>(&self, method: trillium::Method, t: T) -> Option<T> {
        if self.is_allowed(method, &t) {
            Some(t)
        } else {
            None
        }
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
            conn.set_state(actor.clone());
        }

        actor
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
