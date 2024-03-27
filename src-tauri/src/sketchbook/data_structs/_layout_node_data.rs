use crate::sketchbook::layout::LayoutNode;
use crate::sketchbook::{JsonSerde, LayoutId, VarId};
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

impl<'de> JsonSerde<'de> for LayoutNodeData {}

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
        LayoutNodeData::new(
            layout_id.as_str(),
            var_id.as_str(),
            node.get_px(),
            node.get_py(),
        )
    }
}
