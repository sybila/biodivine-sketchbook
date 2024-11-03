use crate::sketchbook::layout::NodePosition;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

/// A node for a particular `Variable`'s in a particular `Layout`. This structure only
/// holds information about the node itself, the id of the variable and layout is held
/// elsewhere (see [super::Layout] or [crate::sketchbook::model::ModelState]).
///
/// Currently, only the position is stored.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutNode {
    position: NodePosition,
    // TODO: add other data in future (visibility, colour, shape, ...)
}

impl LayoutNode {
    /// Create new `LayoutNode` of the regulatory graph.
    pub fn new(px: f32, py: f32) -> LayoutNode {
        LayoutNode {
            position: NodePosition(px, py),
        }
    }

    /// Get the pair of coordinates.
    pub fn get_position(&self) -> &NodePosition {
        &self.position
    }

    /// Get the first (x) coordinate.
    pub fn get_px(&self) -> f32 {
        self.position.0
    }

    /// Get the second (y) coordinate.
    pub fn get_py(&self) -> f32 {
        self.position.1
    }
}

/// Methods for changing node's layout data.
impl LayoutNode {
    /// Change node's coordinates.
    pub fn change_position(&mut self, new_px: f32, new_py: f32) {
        self.position.0 = new_px;
        self.position.1 = new_py;
    }
}

impl Display for LayoutNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Node at [{}, {}]", self.position.0, self.position.1)
    }
}

impl Default for LayoutNode {
    /// Generate new `LayoutNode` at (0,0).
    fn default() -> LayoutNode {
        LayoutNode::new(0., 0.)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::layout::{LayoutNode, NodePosition};

    #[test]
    fn test_layout_node() {
        let mut node = LayoutNode::new(0., 1.);
        assert_eq!(
            node.get_position(),
            &NodePosition(node.get_px(), node.get_py())
        );

        node.change_position(0., 0.);
        let same_node = LayoutNode::default();
        assert_eq!(node, same_node);
    }
}
