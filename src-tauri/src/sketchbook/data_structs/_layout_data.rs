use crate::sketchbook::data_structs::LayoutNodeData;
use crate::sketchbook::ids::LayoutId;
use crate::sketchbook::layout::Layout;
use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};

/// Structure for sending simplified general data about `Layout` to the frontend.
///
/// `LayoutData` does not have the exact same fields as `Layout` (for instance, `id` is added).
/// Some fields of `LayoutData` are simplified compared to `Layout` (e.g., pure `Strings` instead
/// of more complex typesafe structs) to allow for easier (de)serialization.
///
/// See also [LayoutNodeData] for a sub-structure to carry data regarding individual `NodeLayouts`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutData {
    pub id: String,
    pub name: String,
    pub nodes: Vec<LayoutNodeData>,
}

/// Structure for sending *metadata* about `Layout`. This includes id, and name,
/// but excludes all actual nodes.
///
/// Some fields of `LayoutData` are simplified compared to `Layout` (e.g., pure `Strings` instead
/// of more complex typesafe structs) to allow for easier (de)serialization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutMetaData {
    pub id: String,
    pub name: String,
}

impl JsonSerde<'_> for LayoutData {}
impl JsonSerde<'_> for LayoutMetaData {}

impl LayoutData {
    /// Create new `LayoutData` instance given a `layout` and its id.
    pub fn from_layout(layout_id: &LayoutId, layout: &Layout) -> LayoutData {
        let nodes = layout
            .layout_nodes()
            .map(|(v_id, node)| LayoutNodeData::from_node(layout_id, v_id, node))
            .collect();
        LayoutData {
            id: layout_id.to_string(),
            name: layout.get_layout_name().to_string(),
            nodes,
        }
    }

    /// Extract new `Layout` instance from this data.
    pub fn to_layout(&self) -> Result<Layout, String> {
        let var_node_pairs = self
            .nodes
            .iter()
            .map(|node_data| (node_data.variable.as_str(), node_data.to_node()))
            .collect();
        Layout::new(&self.name, var_node_pairs)
    }
}

impl LayoutMetaData {
    /// Create new `LayoutMetaData` instance given a layout's name and id string slices.
    pub fn new(layout_id: &str, layout_name: &str) -> LayoutMetaData {
        LayoutMetaData {
            id: layout_id.to_string(),
            name: layout_name.to_string(),
        }
    }

    /// Create new `LayoutMetaData` instance given a `layout` and its id.
    pub fn from_layout(layout_id: &LayoutId, layout: &Layout) -> LayoutMetaData {
        LayoutMetaData::new(layout_id.as_str(), layout.get_layout_name())
    }
}
