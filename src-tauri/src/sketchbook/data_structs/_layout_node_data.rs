use crate::sketchbook::ids::{LayoutId, VarId};
use crate::sketchbook::layout::LayoutNode;
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

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

/// The same as `LayoutNodeData`, but does not have a fixed variable ID because
/// it is associated with a variable that does not have an ID yet.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutNodeDataPrototype {
    pub layout: String,
    pub px: f32,
    pub py: f32,
}

impl<'de> JsonSerde<'de> for LayoutNodeData {}
impl<'de> JsonSerde<'de> for LayoutNodeDataPrototype {}

impl LayoutNodeData {
    /// Create new `LayoutNodeData` instance given a node's layout ID, variable ID, and coordinates.
    pub fn new(layout_id: &str, var_id: &str, px: f32, py: f32) -> LayoutNodeData {
        LayoutNodeData {
            layout: layout_id.to_string(),
            variable: var_id.to_string(),
            px,
            py,
        }
    }

    /// Create new `LayoutNodeData` instance given a node's layout ID, variable ID,
    /// and corresponding `LayoutNode`.
    pub fn from_node(layout_id: &LayoutId, var_id: &VarId, node: &LayoutNode) -> LayoutNodeData {
        LayoutNodeData::new(
            layout_id.as_str(),
            var_id.as_str(),
            node.get_px(),
            node.get_py(),
        )
    }

    /// Extract new `LayoutNode` instance from this data.
    pub fn to_node(&self) -> LayoutNode {
        LayoutNode::new(self.px, self.py)
    }
}
