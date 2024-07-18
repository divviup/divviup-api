use rand::{distributions::Standard, prelude::Distribution};
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum Protocol {
    #[sea_orm(string_value = "DAP-09")]
    #[serde(rename = "DAP-09")]
    Dap09,
}

impl Distribution<Protocol> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, _rng: &mut R) -> Protocol {
        Protocol::Dap09
    }
}

impl Display for Protocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl AsRef<str> for Protocol {
    fn as_ref(&self) -> &str {
        match self {
            Self::Dap09 => "DAP-09",
        }
    }
}

#[derive(Debug)]
pub struct UnrecognizedProtocol(String);

impl Display for UnrecognizedProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{} was not a recognized protocol", self.0))
    }
}

impl Error for UnrecognizedProtocol {}

impl FromStr for Protocol {
    type Err = UnrecognizedProtocol;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*s.to_lowercase() {
            "dap-09" => Ok(Self::Dap09),
            unrecognized => Err(UnrecognizedProtocol(unrecognized.to_string())),
        }
    }
}
