use crate::sketchbook::ids::{LayoutId, UninterpretedFnId, VarId};
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

/// Structure for sending data about changes in object's ID to the frontend.
///
/// The structure is "generic" and can handle different id types like `VarId` or
/// `UninterpretedFnId`.
///
/// Field `metadata` can be used to carry some additional information (like id of
/// parent component). It is usually empty.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChangeIdData {
    pub original_id: String,
    pub new_id: String,
    #[serde(default)]
    pub metadata: String,
}

impl<'de> JsonSerde<'de> for ChangeIdData {}

impl ChangeIdData {
    /// Create new `ChangeIdData` object given the two id slices and metadata.
    pub fn new_with_metadata(original_id: &str, new_id: &str, metadata: &str) -> ChangeIdData {
        ChangeIdData {
            original_id: original_id.to_string(),
            new_id: new_id.to_string(),
            metadata: metadata.to_string(),
        }
    }

    /// Create new `ChangeIdData` object given the two id slices.
    /// Metadata are empty.
    pub fn new(original_id: &str, new_id: &str) -> ChangeIdData {
        Self::new_with_metadata(original_id, new_id, "")
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
