use crate::sketchbook::layout::NodeLayout;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

/// Structure for sending simplified data about `Layout` to frontend.
/// Only contains some fields, in string format, to allow for simpler parsing and manipulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutNodeData {
    pub var_id: String,
    pub px: f32,
    pub py: f32,
}

impl LayoutNodeData {
    pub fn new(var_id: String, px: f32, py: f32) -> LayoutNodeData {
        LayoutNodeData { var_id, px, py }
    }

    pub fn from_node(id: String, node: &NodeLayout) -> LayoutNodeData {
        LayoutNodeData {
            var_id: id,
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
