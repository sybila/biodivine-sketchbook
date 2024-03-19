use crate::sketchbook::layout::LayoutNode;
use crate::sketchbook::{LayoutId, VarId};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Structure for sending data about `NodeLayout` to frontend.
///
/// Some fields of `LayoutNodeData` are simplified compared to `NodeLayout` (e.g., pure `Strings`
/// instead of more complex typesafe structs) to allow for easier (de)serialization.
///
/// - `layout` is a string ID of the node's layout
/// - `variable` is a string ID of the node's variable
/// - `px` and `py` are the node's coordinates
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutNodeData {
    pub layout: String,
    pub variable: String,
    pub px: f32,
    pub py: f32,
}

impl LayoutNodeData {
    pub fn new(layout_id: &str, var_id: &str, px: f32, py: f32) -> LayoutNodeData {
        LayoutNodeData {
            layout: layout_id.to_string(),
            variable: var_id.to_string(),
            px,
            py,
        }
    }

    pub fn from_node(layout_id: &LayoutId, var_id: &VarId, node: &LayoutNode) -> LayoutNodeData {
        LayoutNodeData {
            layout: layout_id.to_string(),
            variable: var_id.to_string(),
            px: node.get_px(),
            py: node.get_py(),
        }
    }
}

impl Display for LayoutNodeData {
    /// Use json serialization to convert `LayoutNodeData` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for LayoutNodeData {
    type Err = String;

    /// Use json de-serialization to construct `LayoutNodeData` from string.
    fn from_str(s: &str) -> Result<LayoutNodeData, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
