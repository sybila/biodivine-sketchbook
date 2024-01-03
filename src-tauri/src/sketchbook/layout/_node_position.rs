use serde::{Deserialize, Serialize};

/// Position of a particular `LayoutNode` at the editor's layout.
/// Coordinates are floating point and can be negative.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodePosition(pub f32, pub f32);
