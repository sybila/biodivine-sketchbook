use crate::sketchbook::layout::{Layout, LayoutNodeIterator, NodeLayout, NodePosition};
use crate::sketchbook::VarId;

use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Methods for safely constructing or mutating instances of `Layout`.
impl Layout {
    /// Create new empty `Layout` with a given name.
    pub fn new(name_str: &str) -> Layout {
        Layout {
            name: name_str.to_string(),
            nodes: HashMap::new(),
        }
    }

    /// Create new `Layout` with a given name, that is a direct copy of another existing
    /// `template_layout`.
    pub fn new_from_another_copy(name_str: &str, template_layout: &Layout) -> Layout {
        Layout {
            name: name_str.to_string(),
            nodes: template_layout.nodes.clone(),
        }
    }

    /// Create new `Layout` with a given name, which will contain nodes for the same variables as
    /// `template_layout` has, but all of them at a default position.
    pub fn new_from_another_default(name_str: &str, template_layout: &Layout) -> Layout {
        let mut layout = Layout::new(name_str);
        for var_id in template_layout.nodes.keys() {
            // this will never fail if `template_layout` is a valid `Layout`
            layout.add_default_node(var_id.clone()).unwrap();
        }
        layout
    }

    /// Rename this `Layout`.
    pub fn set_layout_name(&mut self, name_str: &str) {
        self.name = name_str.to_string();
    }

    /// Add new node with layout data for a given variable. Currently, nodes only hold information
    /// regarding a 2D position.
    ///
    /// You must ensure that the `variable` is valid before adding it to the layout.
    ///
    /// Returns `Err` if there already is a node for this variable.
    pub fn add_node(&mut self, variable: VarId, p_x: f32, p_y: f32) -> Result<(), String> {
        if self.nodes.contains_key(&variable) {
            return Err(format!("Layout data for {variable} already exist."));
        }
        self.nodes.insert(variable, NodeLayout::new(p_x, p_y));
        Ok(())
    }

    /// Add new default node (at 0,0) for a given variable.
    /// You must ensure that the `variable` is valid before adding it to the layout.
    ///
    /// Returns `Err` if there already is a node for this variable.
    pub fn add_default_node(&mut self, variable: VarId) -> Result<(), String> {
        if self.nodes.contains_key(&variable) {
            return Err(format!("Layout data for {variable} already exist."));
        }
        self.nodes.insert(variable, NodeLayout::default());
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
        self.nodes
            .get_mut(variable)
            .ok_or(format!(
                "Variable {variable} doesn't have a layout information to remove."
            ))?
            .change_position(new_x, new_y);
        Ok(())
    }

    /// Remove a node for given variable from this layout.
    ///
    /// Return `Err` if variable did not have a corresponding node in this layout.
    pub fn remove_node(&mut self, variable: &VarId) -> Result<(), String> {
        if self.nodes.remove(variable).is_none() {
            return Err(format!(
                "Variable {variable} doesn't have a layout information to remove."
            ));
        }
        Ok(())
    }

    /// Set the id of variable with `original_id` to `new_id`.
    pub fn change_node_id(&mut self, original_id: &VarId, new_id: VarId) -> Result<(), String> {
        if let Some(node_layout) = self.nodes.remove(original_id) {
            self.nodes.insert(new_id.clone(), node_layout);
        } else {
            return Err(format!(
                "Variable {original_id} doesn't have a layout information to remove."
            ));
        }
        Ok(())
    }
}

/// Methods for observing instances of `ModelState` (various getters, etc.).
impl Layout {
    /// Layout information regarding the node for a particular variable.
    pub fn get_node(&self, variable: &VarId) -> Result<&NodeLayout, String> {
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
    pub fn layout_nodes(&self) -> LayoutNodeIterator {
        self.nodes.iter()
    }
}

impl Display for Layout {
    /// Use json serialization to convert `Layout` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for Layout {
    type Err = String;

    /// Use json de-serialization to construct `Layout` from string.
    fn from_str(s: &str) -> Result<Layout, <Layout as FromStr>::Err> {
        match serde_json::from_str(s) {
            Ok(layout) => Ok(layout),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::layout::{Layout, NodeLayout};
    use crate::sketchbook::VarId;

    #[test]
    fn test_layout_basics() {
        // playing with layout name
        let mut layout = Layout::new("fancy_name");
        assert_eq!(layout.get_layout_name(), "fancy_name");
        layout.set_layout_name("name_v2");
        assert_eq!(layout.get_layout_name(), "name_v2");

        // predefine few var IDs and expected nodes for later
        let var_id1 = VarId::new("v1").unwrap();
        let var_id2 = VarId::new("v2").unwrap();
        let var_id1_again = VarId::new("v1").unwrap();
        let var_id3 = VarId::new("v3").unwrap();
        let default_node = NodeLayout::default();
        let node_ten_ten = NodeLayout::new(10., 10.);

        // add node v1, node v2, and try adding v1 again (should fail)
        layout.add_default_node(var_id1.clone()).unwrap();
        assert_eq!(layout.get_node(&var_id1).unwrap(), &default_node);
        layout.add_node(var_id2.clone(), 1., 2.).unwrap();
        assert!(layout.add_node(var_id1_again, 1., 2.).is_err());
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
        let mut layout = Layout::new("fancy_name");
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
