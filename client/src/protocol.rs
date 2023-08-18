use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    #[serde(rename = "DAP-04")]
    Dap04,
    #[serde(rename = "DAP-05")]
    Dap05,
}

impl AsRef<str> for Protocol {
    fn as_ref(&self) -> &str {
        match self {
            Self::Dap04 => "DAP-04",
            Self::Dap05 => "DAP-05",
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
            "dap-04" => Ok(Self::Dap04),
            "dap-05" => Ok(Self::Dap05),
            unrecognized => Err(UnrecognizedProtocol(unrecognized.to_string())),
        }
    }
}
