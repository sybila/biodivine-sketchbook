use crate::sketchbook::layout::NodePosition;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

/// Layout information for a particular `Variable`'s node.
/// Currently, only the position is stored.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeLayout {
    position: NodePosition,
    // todo: add other data (visibility, colour, shape, ...)
}

impl NodeLayout {
    /// Create a new layout data for a node of the regulatory graph.
    pub fn new(px: f32, py: f32) -> NodeLayout {
        NodeLayout {
            position: NodePosition(px, py),
        }
    }

    /// Getter for the pair of coordinates.
    pub fn get_position(&self) -> &NodePosition {
        &self.position
    }

    /// Getter for first (x) coordinate.
    pub fn get_px(&self) -> f32 {
        self.position.0
    }

    /// Getter for second (y) coordinate.
    pub fn get_py(&self) -> f32 {
        self.position.1
    }
}

/// Methods for changing node's layout data.
impl NodeLayout {
    pub fn change_position(&mut self, new_px: f32, new_py: f32) {
        self.position.0 = new_px;
        self.position.1 = new_py;
    }
}

impl Display for NodeLayout {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Node at [{}, {}]", self.position.0, self.position.1)
    }
}

impl Default for NodeLayout {
    fn default() -> NodeLayout {
        NodeLayout::new(0., 0.)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::layout::{NodeLayout, NodePosition};

    #[test]
    fn test_layout_node() {
        let mut node = NodeLayout::new(0., 1.);
        assert_eq!(
            node.get_position(),
            &NodePosition(node.get_px(), node.get_py())
        );

        node.change_position(0., 0.);
        let same_node = NodeLayout::default();
        assert_eq!(node, same_node);
    }
}
