use serde::{Deserialize, Serialize};

/// Position of the `Variable`'s node at the editor's layout.
/// Coordinates can are floating point and can be negative.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodePosition(pub f32, pub f32);
