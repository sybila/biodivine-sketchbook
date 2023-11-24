use crate::sketchbook::{Identifier, VarId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// **(internal)** Utility methods for `Layout`.
mod _impl_layout;
/// **(internal)**  Utility methods for `LayoutId`.
mod _impl_layout_id;
/// **(internal)** Implementation of [Serialize] and [Deserialize] traits for `Layout`.
mod _impl_layout_serde;
/// **(internal)**  Utility methods for `NodeLayout`.
mod _impl_node_layout;

/// A type-safe (string-based) identifier of a `Layout` inside `RegulationsState`.
///
/// **Warning:** Do not mix identifiers between different networks/graphs. Generally, be careful
/// to only use `LayoutId` currently valid for the network.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct LayoutId {
    id: Identifier,
}

/// Position of the `Variable`'s node at the editor's layout.
/// Coordinates can are floating point and can be negative.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodePosition(pub f32, pub f32);

/// Layout information for a particular `Variable`'s node.
/// Currently, only the position is stored.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeLayout {
    position: NodePosition,
    // todo: add other data (visibility, colour, shape, ...)
}

/// Structure to capture all the layout data regarding one particular layout of the regulations
/// editor.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    name: String,
    nodes: HashMap<VarId, NodeLayout>,
    // todo: add compartments
}
