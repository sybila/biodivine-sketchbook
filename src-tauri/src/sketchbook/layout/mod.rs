use crate::sketchbook::VarId;

/// **(internal)** Utility methods for `Layout`.
mod _layout;
/// **(internal)**  Utility methods for `LayoutId`.
mod _layout_id;
/// **(internal)**  Utility methods for `NodeLayout`.
mod _node_layout;
/// **(internal)**  Utility methods for `NodePosition`.
mod _node_position;

pub use _layout::Layout;
pub use _layout_id::LayoutId;
pub use _node_layout::NodeLayout;
pub use _node_position::NodePosition;

/// An iterator over all (`VarId`, `NodeLayout`) pairs of a `Layout`.
pub type LayoutNodeIterator<'a> = std::collections::hash_map::Iter<'a, VarId, NodeLayout>;
