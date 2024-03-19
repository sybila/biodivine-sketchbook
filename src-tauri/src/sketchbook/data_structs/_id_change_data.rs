use crate::sketchbook::{LayoutId, UninterpretedFnId, VarId};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Structure for sending data about changes in object's ID to the frontend.
///
/// The structure is "generic" and can handle different id types like `VarId` or
/// `UninterpretedFnId`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChangeIdData {
    pub original_id: String,
    pub new_id: String,
}

impl ChangeIdData {
    /// Create new `ChangeIdData` object given the two id slices.
    pub fn new(original_id: &str, new_id: &str) -> ChangeIdData {
        ChangeIdData {
            original_id: original_id.to_string(),
            new_id: new_id.to_string(),
        }
    }

    /// Create new `ChangeIdData` object given variable IDs.
    pub fn from_var_id(original_id: &VarId, new_id: &VarId) -> ChangeIdData {
        Self::new(original_id.as_str(), new_id.as_str())
    }

    /// Create new `ChangeIdData` object given layout IDs.
    pub fn from_layout_id(original_id: &LayoutId, new_id: &LayoutId) -> ChangeIdData {
        Self::new(original_id.as_str(), new_id.as_str())
    }

    /// Create new `ChangeIdData` object given uninterpreted fn IDs.
    pub fn from_fn_id(original_id: &UninterpretedFnId, new_id: &UninterpretedFnId) -> ChangeIdData {
        Self::new(original_id.as_str(), new_id.as_str())
    }
}

impl Display for ChangeIdData {
    /// Use json serialization to convert `ChangeIdData` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for ChangeIdData {
    type Err = String;

    /// Use json de-serialization to construct `ChangeIdData` from string.
    fn from_str(s: &str) -> Result<ChangeIdData, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
