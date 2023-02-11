use std::borrow::Cow;

use trillium::{Conn, Handler, KnownHeaderName::Location, Status};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum RedirectStatus {
    MultipleChoices,
    MovedPermanently,
    #[default]
    Found,
    SeeOther,
    TemporaryRedirect,
    PermanentRedirect,
}

impl From<RedirectStatus> for Status {
    fn from(value: RedirectStatus) -> Self {
        match value {
            RedirectStatus::MultipleChoices => Status::MultipleChoice,
            RedirectStatus::MovedPermanently => Status::MovedPermanently,
            RedirectStatus::Found => Status::Found,
            RedirectStatus::SeeOther => Status::SeeOther,
            RedirectStatus::TemporaryRedirect => Status::TemporaryRedirect,
            RedirectStatus::PermanentRedirect => Status::PermanentRedirect,
        }
    }
}

pub struct Redirect {
    to: Cow<'static, str>,
    status: RedirectStatus,
}

impl Redirect {
    pub fn to(to: impl Into<Cow<'static, str>>) -> Self {
        Self {
            to: to.into(),
            status: RedirectStatus::default(),
        }
    }

    pub fn with_redirect_status(mut self, status: RedirectStatus) -> Self {
        self.status = status.into();
        self
    }
}

#[trillium::async_trait]
impl Handler for Redirect {
    async fn run(&self, conn: Conn) -> Conn {
        conn.with_status(self.status)
            .with_header(Location, self.to.to_string())
            .halt()
    }
}
