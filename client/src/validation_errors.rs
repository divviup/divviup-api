use pad_adapter::PadAdapter;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter, Write},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ValidationError {
    code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    params: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ValidationErrors {
    Map(HashMap<String, ValidationErrors>),
    List(Vec<ValidationError>),
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let code = &self.code;
        match (&self.message, self.params.is_empty()) {
            (None, true) => f.write_str(&self.code),
            (Some(message), _) => f.write_fmt(format_args!("{code} {message}")),
            (None, false) => f.write_fmt(format_args!(
                "{code} {}",
                serde_json::to_string(&self.params).unwrap()
            )),
        }
    }
}

impl Display for ValidationErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut pad_adapter = PadAdapter::with_padding(f, "  ");
        match &self {
            ValidationErrors::Map(map) => {
                for (key, values) in map {
                    write!(pad_adapter, "- {key}:\n{values}")?;
                }
            }
            ValidationErrors::List(errors) => {
                for value in errors {
                    write!(pad_adapter, "* {value}\n")?;
                }
            }
        }

        Ok(())
    }
}
