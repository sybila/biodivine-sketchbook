use crate::sketchbook::layout::NodeLayout;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

/// Structure for sending data about `NodeLayout` to frontend.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutNodeData {
    pub layout: String,
    pub variable: String,
    pub px: f32,
    pub py: f32,
}

impl LayoutNodeData {
    pub fn new(layout_id: String, var_id: String, px: f32, py: f32) -> LayoutNodeData {
        LayoutNodeData { layout: layout_id, variable: var_id, px, py }
    }

    pub fn from_node(layout_id: String, var_id: String, node: &NodeLayout) -> LayoutNodeData {
        LayoutNodeData {
            layout: layout_id,
            variable: var_id,
            px: node.get_px(),
            py: node.get_py(),
        }
    }
}

impl Display for LayoutNodeData {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
