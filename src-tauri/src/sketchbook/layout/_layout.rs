use crate::sketchbook::ids::VarId;
use crate::sketchbook::layout::{LayoutNode, LayoutNodeIterator, NodePosition};
use crate::sketchbook::utils::{assert_ids_unique, assert_name_valid};
use crate::sketchbook::Manager;
use std::collections::HashMap;

/// Structure to capture all the layout data regarding one particular layout of the regulations
/// editor.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    name: String,
    nodes: HashMap<VarId, LayoutNode>,
}

impl Manager for Layout {}

/// Methods for safely constructing or mutating instances of `Layout`.
impl Layout {
    /// Create new empty `Layout` (i.e., with no nodes) with a given name.
    pub fn new_empty(name: &str) -> Result<Layout, String> {
        assert_name_valid(name)?;
        Ok(Layout {
            name: name.to_string(),
            nodes: HashMap::new(),
        })
    }

    /// Create new `Layout` with a given name and nodes.
    pub fn new(name: &str, var_node_pairs: Vec<(&str, LayoutNode)>) -> Result<Layout, String> {
        // before making any changes, check that all IDs are actually valid and unique
        let var_ids: Vec<&str> = var_node_pairs.iter().map(|pair| pair.0).collect();
        assert_ids_unique(&var_ids)?;

        // now we can safely add them
        let mut layout = Layout::new_empty(name)?;
        for (var_id, node) in var_node_pairs {
            layout.add_node(VarId::new(var_id)?, node)?;
        }
        Ok(layout)
    }

    /// Create new `Layout` with a given name, that is a direct copy of another existing
    /// valid `template_layout`.
    pub fn new_from_another_copy(name: &str, template_layout: &Layout) -> Layout {
        Layout {
            name: name.to_string(),
            nodes: template_layout.nodes.clone(),
        }
    }

    /// Create new `Layout` with a given name, which will contain nodes all the given variables,
    /// all of the nodes will be located at a default position.
    ///
    /// Returns `Error` if given ids contain duplicates.
    pub fn new_from_vars_default(name: &str, variables: Vec<VarId>) -> Result<Layout, String> {
        // before making any changes, check that all IDs are actually valid and unique
        assert_ids_unique(&variables)?;
        // now we can safely add them
        let mut layout = Layout::new_empty(name)?;
        for var_id in variables {
            layout.add_default_node(var_id.clone())?;
        }
        Ok(layout)
    }

    /// Rename this `Layout`.
    pub fn set_layout_name(&mut self, name: &str) -> Result<(), String> {
        assert_name_valid(name)?;
        self.name = name.to_string();
        Ok(())
    }

    /// Add a new (pre-generated) node.
    ///
    /// You must ensure that the `variable` is valid before adding it to the layout.
    ///
    /// Returns `Err` if there already is a node for this variable.
    pub fn add_node(&mut self, var: VarId, node: LayoutNode) -> Result<(), String> {
        self.assert_no_variable(&var)?;
        self.nodes.insert(var, node);
        Ok(())
    }

    /// Add a new node for a given variable to this layout. Currently, nodes only hold information
    /// regarding a 2D position.
    ///
    /// You must ensure that the `variable` is valid before adding it to the layout.
    ///
    /// Returns `Err` if there already is a node for this variable.
    pub fn add_node_by_coords(&mut self, var: VarId, p_x: f32, p_y: f32) -> Result<(), String> {
        self.assert_no_variable(&var)?;
        self.nodes.insert(var, LayoutNode::new(p_x, p_y));
        Ok(())
    }

    /// Add a new default node (at 0,0) for a given variable.
    ///
    /// You must ensure that the `variable` is valid before adding it to the layout.
    ///
    /// Returns `Err` if there already is a node for this variable.
    pub fn add_default_node(&mut self, variable: VarId) -> Result<(), String> {
        self.assert_no_variable(&variable)?;
        self.nodes.insert(variable, LayoutNode::default());
        Ok(())
    }

    /// Update position of a node for a given variable.
    ///
    /// Return `Err` if variable did not have a corresponding node in this layout.
    pub fn update_node_position(
        &mut self,
        variable: &VarId,
        new_x: f32,
        new_y: f32,
    ) -> Result<(), String> {
        self.assert_valid_variable(variable)?;
        self.nodes
            .get_mut(variable)
            .unwrap()
            .change_position(new_x, new_y);
        Ok(())
    }

    /// Remove a node for a given variable from this layout.
    ///
    /// Return `Err` if variable did not have a corresponding node in this layout.
    pub fn remove_node(&mut self, variable: &VarId) -> Result<(), String> {
        self.assert_valid_variable(variable)?;
        self.nodes.remove(variable);
        Ok(())
    }

    /// Change id of a variable with `original_id` to `new_id`.
    pub fn change_node_id(&mut self, original_id: &VarId, new_id: VarId) -> Result<(), String> {
        self.assert_valid_variable(original_id)?;
        if let Some(node_layout) = self.nodes.remove(original_id) {
            self.nodes.insert(new_id.clone(), node_layout);
        }
        Ok(())
    }
}

/// Utility methods to assert (non-)existence of nodes in the layout.
impl Layout {
    /// **(internal)** Utility method to ensure there is no node for the variable with given Id yet.
    fn assert_no_variable(&self, var_id: &VarId) -> Result<(), String> {
        if self.nodes.contains_key(var_id) {
            Err(format!("Layout node for {var_id} already exists."))
        } else {
            Ok(())
        }
    }

    /// **(internal)** Utility method to ensure there is a node for a variable with given Id.
    fn assert_valid_variable(&self, var_id: &VarId) -> Result<(), String> {
        if self.nodes.contains_key(var_id) {
            Ok(())
        } else {
            Err(format!("Layout node for {var_id} does not exist."))
        }
    }
}

/// Methods for observing instances of `ModelState` (various getters, etc.).
impl Layout {
    /// Layout information regarding the node for a particular variable.
    pub fn get_node(&self, variable: &VarId) -> Result<&LayoutNode, String> {
        self.nodes
            .get(variable)
            .ok_or(format!("No layout data for variable {variable}."))
    }

    /// Human-readable name of this layout.
    pub fn get_layout_name(&self) -> &String {
        &self.name
    }

    /// Number of nodes in this layout.
    pub fn get_node_position(&self, variable: &VarId) -> Result<&NodePosition, String> {
        Ok(self.get_node(variable)?.get_position())
    }

    /// Number of nodes in this layout.
    pub fn get_num_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Return an iterator over all nodes of this layout.
    pub fn layout_nodes(&self) -> LayoutNodeIterator<'_> {
        self.nodes.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::ids::VarId;
    use crate::sketchbook::layout::{Layout, LayoutNode};

    #[test]
    fn test_layout_basics() {
        // playing with layout name
        let mut layout = Layout::new_empty("fancy_name").unwrap();
        assert_eq!(layout.get_layout_name(), "fancy_name");
        layout.set_layout_name("name_v2").unwrap();
        assert_eq!(layout.get_layout_name(), "name_v2");

        // predefine few var IDs and expected nodes for later
        let var_id1 = VarId::new("v1").unwrap();
        let var_id2 = VarId::new("v2").unwrap();
        let var_id1_again = VarId::new("v1").unwrap();
        let var_id3 = VarId::new("v3").unwrap();
        let default_node = LayoutNode::default();
        let node_ten_ten = LayoutNode::new(10., 10.);

        // add node v1, node v2, and try adding v1 again (should fail)
        layout.add_default_node(var_id1.clone()).unwrap();
        assert_eq!(layout.get_node(&var_id1).unwrap(), &default_node);
        layout.add_node_by_coords(var_id2.clone(), 1., 2.).unwrap();
        assert!(layout.add_node_by_coords(var_id1_again, 1., 2.).is_err());
        assert_eq!(layout.get_num_nodes(), 2);

        // change position of node v1, and try changing position of node thats not in the network
        layout.update_node_position(&var_id1, 10., 10.).unwrap();
        assert_eq!(layout.get_node(&var_id1).unwrap(), &node_ten_ten);
        assert!(layout.update_node_position(&var_id3, 10., 10.).is_err());

        // remove node v1, and check that its not longer present
        layout.remove_node(&var_id1).unwrap();
        assert!(layout.get_node(&var_id1).is_err());
        assert_eq!(layout.get_num_nodes(), 1);
    }

    #[test]
    fn test_new_layout_from() {
        let mut layout = Layout::new_empty("fancy_name").unwrap();
        let var_id1 = VarId::new("v1").unwrap();
        layout.add_default_node(var_id1.clone()).unwrap();

        // make new layout as a copy of this one
        let layout_2 = Layout::new_from_another_copy("new_one", &layout);
        assert_eq!(layout_2.get_num_nodes(), 1);

        // test that changes do not propagate
        layout.remove_node(&var_id1).unwrap();
        assert_eq!(layout.get_num_nodes(), 0);
        assert_eq!(layout_2.get_num_nodes(), 1);
    }
}
