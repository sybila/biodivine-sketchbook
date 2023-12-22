use crate::sketchbook::Variable;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Structure for sending simplified data about `Variable` to frontend.
/// Only contains some fields, in string format, to allow for simpler parsing and manipulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariableData {
    pub id: String,
    pub name: String,
}

impl VariableData {
    pub fn new(id: String, name: String) -> VariableData {
        VariableData { id, name }
    }

    pub fn from_var(id: String, variable: &Variable) -> VariableData {
        VariableData {
            id,
            name: variable.get_name().to_string(),
        }
    }
}

impl Display for VariableData {
    /// Use json serialization to convert `VariableData` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for VariableData {
    type Err = String;

    /// Use json de-serialization to construct `Variable` from string.
    fn from_str(s: &str) -> Result<VariableData, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
