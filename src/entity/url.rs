use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    ops::{Deref, DerefMut},
    str::FromStr,
};
use trillium_client::IntoUrl;
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(transparent)]
pub struct Url(url::Url);

impl Display for Url {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Deref for Url {
    type Target = url::Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Url {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<Url> for url::Url {
    fn from(value: Url) -> Self {
        value.0
    }
}
impl From<url::Url> for Url {
    fn from(value: url::Url) -> Self {
        Self(value)
    }
}
impl From<Url> for sea_orm::Value {
    fn from(value: Url) -> Self {
        sea_orm::Value::String(Some(Box::new(value.to_string())))
    }
}
impl PartialEq<str> for Url {
    fn eq(&self, other: &str) -> bool {
        self.0.as_ref() == other
    }
}
impl PartialEq<&str> for Url {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_ref() == *other
    }
}
impl PartialEq<url::Url> for Url {
    fn eq(&self, other: &url::Url) -> bool {
        self.0 == *other
    }
}
impl FromStr for Url {
    type Err = url::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        url::Url::from_str(s).map(Self)
    }
}

impl IntoUrl for Url {
    fn into_url(self, base: Option<&url::Url>) -> trillium_http::Result<url::Url> {
        self.0.into_url(base)
    }
}

impl sea_orm::TryGetable for Url {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        idx: I,
    ) -> Result<Self, sea_orm::TryGetError> {
        let string: String = res.try_get_by(idx).map_err(sea_orm::TryGetError::DbErr)?;
        match url::Url::parse(&string) {
            Ok(url) => Ok(Self(url)),
            Err(e) => Err(sea_orm::TryGetError::DbErr(sea_orm::DbErr::Custom(
                e.to_string(),
            ))),
        }
    }
}

impl sea_orm::sea_query::ValueType for Url {
    fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
        match v {
            sea_orm::Value::String(Some(x)) => url::Url::parse(&x)
                .map_err(|_| sea_orm::sea_query::ValueTypeErr)
                .map(Self),
            _ => Err(sea_orm::sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(Url).to_owned()
    }

    fn array_type() -> sea_orm::sea_query::ArrayType {
        sea_orm::sea_query::ArrayType::String
    }

    fn column_type() -> sea_orm::entity::ColumnType {
        sea_orm::entity::ColumnType::String(None)
    }
}

impl sea_orm::sea_query::Nullable for Url {
    fn null() -> sea_orm::Value {
        String::null()
    }
}
