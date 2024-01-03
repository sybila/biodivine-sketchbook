use crate::sketchbook::{VarId, Variable};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Structure for sending data about `Variable` to the frontend.
///
/// `VariableData` does not have the exact same fields as `Variable` (for instance, there is an additional useful
/// field `id`). All the fields of `VariableData` are string to allow for simpler (de)serialization and manipulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariableData {
    pub id: String,
    pub name: String,
}

impl VariableData {
    /// Create new `VariableData` object given a variable's `name` and `id` string slices.
    pub fn new(id: &str, name: &str) -> VariableData {
        VariableData {
            id: id.to_string(),
            name: name.to_string(),
        }
    }

    /// Create new `VariableData` object given a `variable` and its id.
    pub fn from_var(var_id: &VarId, variable: &Variable) -> VariableData {
        VariableData {
            id: var_id.to_string(),
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
