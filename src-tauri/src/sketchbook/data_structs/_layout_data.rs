use crate::sketchbook::layout::Layout;
use crate::sketchbook::LayoutId;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Structure for sending simplified general data about `Layout` to the frontend.
///
/// `LayoutData` does not have the exact same fields as `Layout` (for instance, there is an additional useful
/// field `id` and the layout nodes are missing).
/// Some fields of `LayoutData` are simplified compared to `Layout` (e.g., pure `Strings` instead
/// of more complex typesafe structs) to allow for easier (de)serialization.
///
/// See also [LayoutNodeData] for similar structure to carry data regarding `NodeLayouts`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutData {
    pub id: String,
    pub name: String,
}

impl LayoutData {
    /// Create new `LayoutData` object given a layout's name and id string slices.
    pub fn new(layout_id: &str, layout_name: &str) -> LayoutData {
        LayoutData {
            id: layout_id.to_string(),
            name: layout_name.to_string(),
        }
    }

    /// Create new `VariableData` object given a `variable` and its id.
    pub fn from_layout(layout_id: &LayoutId, layout: &Layout) -> LayoutData {
        LayoutData::new(layout_id.as_str(), layout.get_layout_name())
    }
}

impl Display for LayoutData {
    /// Use json serialization to convert `LayoutData` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for LayoutData {
    type Err = String;

    /// Use json de-serialization to construct `LayoutData` from string.
    fn from_str(s: &str) -> Result<LayoutData, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
