use crate::sketchbook::layout::Layout;
use crate::sketchbook::{JsonSerde, LayoutId};
use serde::{Deserialize, Serialize};

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

impl<'de> JsonSerde<'de> for LayoutData {}

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
