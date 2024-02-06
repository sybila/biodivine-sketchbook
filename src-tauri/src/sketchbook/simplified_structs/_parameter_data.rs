use crate::sketchbook::{ParamId, Parameter};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Structure for sending data about `Parameter` to the frontend.
///
/// `ParameterData` does not have the exact same fields as `Parameter` (for instance, there is an additional useful
/// field `id`). All the fields of `ParameterData` are string to allow for simpler (de)serialization and manipulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParameterData {
    pub id: String,
    pub name: String,
    pub arity: u32,
}

impl ParameterData {
    /// Create new `ParameterData` object given a parameter's `name`, `arity`, and `id`.
    pub fn new(id: &str, name: &str, arity: u32) -> ParameterData {
        ParameterData {
            id: id.to_string(),
            name: name.to_string(),
            arity,
        }
    }

    /// Create new `ParameterData` object given a `parameter` and its id.
    pub fn from_param(param_id: &ParamId, param: &Parameter) -> ParameterData {
        ParameterData {
            id: param_id.to_string(),
            name: param.get_name().to_string(),
            arity: param.get_arity(),
        }
    }
}

impl Display for ParameterData {
    /// Use json serialization to convert `ParameterData` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for ParameterData {
    type Err = String;

    /// Use json de-serialization to construct `ParameterData` from string.
    fn from_str(s: &str) -> Result<ParameterData, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
