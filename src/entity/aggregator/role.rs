use rand::{distributions::Standard, prelude::Distribution};
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum Role {
    #[sea_orm(num_value = 0)]
    Leader,
    #[sea_orm(num_value = 1)]
    Helper,
    #[sea_orm(num_value = 2)]
    Either,
}

impl Distribution<Role> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Role {
        match rng.gen_range(0..3) {
            0 => Role::Leader,
            1 => Role::Helper,
            _ => Role::Either,
        }
    }
}
impl AsRef<str> for Role {
    fn as_ref(&self) -> &str {
        match self {
            Self::Leader => "leader",
            Self::Helper => "helper",
            Self::Either => "either",
        }
    }
}

#[derive(Debug)]
pub struct UnrecognizedRole(String);
impl Display for UnrecognizedRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{} was not a recognized role option", self.0))
    }
}
impl Error for UnrecognizedRole {}
impl FromStr for Role {
    type Err = UnrecognizedRole;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*s.to_lowercase() {
            "leader" => Ok(Self::Leader),
            "helper" => Ok(Self::Helper),
            "either" => Ok(Self::Either),
            unrecognized => Err(UnrecognizedRole(unrecognized.to_string())),
        }
    }
}
