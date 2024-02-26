use crate::sketchbook::layout::LayoutNode;
use crate::sketchbook::VarId;
use std::collections::HashMap;

/// **(internal)** Basic utility methods for `Layout`.
mod _impl_layout;
/// **(internal)** Implementation of [Serialize] and [Deserialize] traits for `Layout`.
mod _impl_layout_serde;

/// Structure to capture all the layout data regarding one particular layout of the regulations
/// editor.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    name: String,
    nodes: HashMap<VarId, LayoutNode>,
}
