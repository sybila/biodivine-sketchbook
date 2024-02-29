use crate::sketchbook::{UpdateFn, VarId};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Structure for sending data about `UpdateFn` to the frontend.
///
/// `UpdateFnData` does not have the exact same fields as `UpdateFn` (for instance, there
/// is an additional useful field `id`, and the syntactic tree is missing). Some  fields of
/// `UpdateFnData` are simplified to allow for simpler (de)serialization and manipulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UpdateFnData {
    pub var_id: String,
    pub expression: String,
}

impl UpdateFnData {
    /// Create new `UpdateFnData` object given a update function's `expression` and an id of
    /// its corresponding variable.
    pub fn new(var_id: &str, expression: &str) -> UpdateFnData {
        UpdateFnData {
            var_id: var_id.to_string(),
            expression: expression.to_string(),
        }
    }

    /// Create new `UpdateFnData` object given an update function and corresponding variable's id.
    pub fn from_update_fn(var_id: &VarId, update_fn: &UpdateFn) -> UpdateFnData {
        UpdateFnData {
            var_id: var_id.to_string(),
            expression: update_fn.get_fn_expression().to_string(),
        }
    }
}

impl Display for UpdateFnData {
    /// Use json serialization to convert `UpdateFnData` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for UpdateFnData {
    type Err = String;

    /// Use json de-serialization to construct `UpdateFnData` from string.
    fn from_str(s: &str) -> Result<UpdateFnData, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
