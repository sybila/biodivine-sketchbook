use crate::sketchbook::{
    Layout, LayoutId, RegulationSign, Regulation, RegulationsState, VarId, Variable,
};
use std::collections::{HashMap, HashSet};

/// Methods for safely constructing or mutating instances of `RegulationsState`.
///
/// These methods focus on general manipulating with variables/regulations.
/// See below for API focusing on layout manipulation.
impl RegulationsState {
    /// Create a new `RegulationsState` that does not contain any `Variables` or `Regulations`.
    /// It contains a single empty default `Layout`.
    pub fn new() -> RegulationsState {
        let default_layout_id = RegulationsState::get_default_layout_id();
        let default_layout = Layout::new(RegulationsState::get_default_layout_name());
        RegulationsState {
            variables: HashMap::new(),
            regulations: HashSet::new(),
            layouts: HashMap::from([(default_layout_id, default_layout)]),
        }
    }

    /// Create new `RegulationsState` using provided variable name-ID pairs, both strings.
    /// Result will contain no `Regulations`, and a single default `Layout`.
    ///
    /// The IDs must be unique valid identifiers.
    /// The names might be same as the IDs. It also might be empty or non-unique.
    ///
    /// Return `Err` in case the IDs are not unique.
    pub fn new_from_vars(variables: Vec<(&str, &str)>) -> Result<RegulationsState, String> {
        let mut reg_state = RegulationsState::new();

        let var_id_set = variables.iter().map(|pair| pair.0).collect::<HashSet<_>>();
        if var_id_set.len() != variables.len() {
            return Err(format!("Variables {:?} contain duplicates.", variables));
        }

        for (id, var_name) in variables {
            let var_id = VarId::new(id)?;
            reg_state
                .variables
                .insert(var_id.clone(), Variable::new(var_name));
            reg_state.insert_to_default_layout(var_id)?;
        }
        Ok(reg_state)
    }

    /// Add a new variable with given `var_id` and `name` to this `RegulationsState`.
    ///
    /// The ID must be valid identifier that is not already used by some other variable.
    /// The names might be same as the ID. It also might be empty or non-unique.
    ///
    /// Returns `Err` in case the `id` is already being used.
    pub fn add_var(&mut self, var_id: VarId, name: &str) -> Result<(), String> {
        self.assert_no_variable(&var_id)?;
        self.variables.insert(var_id.clone(), Variable::new(name));
        self.insert_to_all_layouts(var_id)?;
        Ok(())
    }

    /// Add a new variable with given `id` and `name` to this `RegulationsState`.
    ///
    /// The ID must be valid identifier that is not already used by some other variable.
    /// The names might be same as the ID. It also might be empty or non-unique.
    ///
    /// Returns `Err` in case the `id` is not a valid identifier or if it is already being used.
    pub fn add_var_by_str(&mut self, id: &str, name: &str) -> Result<(), String> {
        let var_id = VarId::new(id)?;
        self.add_var(var_id, name)
    }

    /// Add a new `Regulation` to this `RegulationsState`.
    ///
    /// Returns `Err` when one of the variables is invalid, or the regulation between the two
    /// variables already exists.
    pub fn add_regulation(
        &mut self,
        regulator: VarId,
        target: VarId,
        observable: bool,
        regulation_sign: RegulationSign,
    ) -> Result<(), String> {
        self.assert_valid_variable(&regulator)?;
        self.assert_valid_variable(&target)?;
        self.assert_no_regulation(&regulator, &target)?;
        self.regulations.insert(Regulation {
            regulator,
            target,
            observable,
            regulation_sign,
        });
        Ok(())
    }

    /// Add a new `Regulation` to this `RegulationsState` using a string representation. The
    /// variables in the given string must be valid ID strings for this `RegulationsState`.
    ///
    /// Returns `Err` when the string does not encode a valid regulation, if the provided variables
    /// are not valid variable IDs, or when the regulation between the two variables already exists.
    pub fn add_regulation_by_str(&mut self, regulation_str: &str) -> Result<(), String> {
        let (reg, regulation_sign, observable, tar) = Regulation::try_from_string(regulation_str)?;
        let regulator = VarId::new(reg.as_str())?;
        let target = VarId::new(tar.as_str())?;
        self.add_regulation(regulator, target, observable, regulation_sign)
    }

    /// Set the name of a network variable given by id `var_id`.
    ///
    /// The name does not have to be unique, as multiple variables might share a name.
    ///
    /// Note that you don't have to rename anything else in the network, since all other
    /// structures reference variables with ids.
    pub fn set_var_name(&mut self, var_id: &VarId, name: &str) -> Result<(), String> {
        self.assert_valid_variable(var_id)?;
        let variable = self.variables.get_mut(var_id).unwrap();
        let mut name_string = name.to_string();
        std::mem::swap(&mut name_string, &mut variable.name);
        Ok(())
    }

    /// Set the name of a network variable given by string `id`.
    ///
    /// The name does not have to be unique, as multiple variables might share a name.
    pub fn set_var_name_by_str(&mut self, id: &str, name: &str) -> Result<(), String> {
        let var_id = VarId::new(id)?;
        self.set_var_name(&var_id, name)
    }

    /// Remove the network variable with given `var_id` from this `RegulationsState`. This also
    /// removes the variable from all `Layouts` and removes all `Regulations` where this
    /// variable figures.
    ///
    /// Returns `Err` in case the `var_id` is not a valid variable's identifier.
    pub fn remove_var(&mut self, var_id: &VarId) -> Result<(), String> {
        self.assert_valid_variable(var_id)?;
        // first delete all regulations, layout nodes, and lastly the variable itself
        self.remove_all_regulations_var(var_id)?;
        self.remove_from_all_layouts(var_id)?;
        if self.variables.remove(var_id).is_none() {
            panic!("Error when removing variable {var_id} from the variable map.")
        }

        Ok(())
    }

    /// Remove the network variable with given `id` from this `RegulationsState`. This also
    /// removes the variable from all `Layouts` and removes all `Regulations` where this
    /// variable figures.
    ///
    /// Returns `Err` in case the `var_id` is not a valid variable's identifier.
    pub fn remove_var_by_str(&mut self, id: &str) -> Result<(), String> {
        let var_id = VarId::new(id)?;
        self.remove_var(&var_id)
    }

    /// Remove a `Regulation` pointing from `regulator` to `target` from this `RegulationsState`.
    ///
    /// Returns `Err` when one of the variables is invalid, or the regulation between the two
    /// variables does not exist.
    pub fn remove_regulation(&mut self, regulator: &VarId, target: &VarId) -> Result<(), String> {
        // all validity checks are performed inside
        let regulation = self.get_regulation(regulator, target)?.clone();
        if !self.regulations.remove(&regulation) {
            panic!("Error when removing regulation between {regulator} - {target}.")
        }
        Ok(())
    }

    /// Remove a `Regulation` given by `regulation_str` from this `RegulationsState`. The
    /// variables in the `regulation_str` must be valid ID strings for this `RegulationsState`.
    ///
    /// Returns `Err` when one of the variables is invalid, or the regulation between the two
    /// variables does not exist.
    pub fn remove_regulation_by_str(&mut self, regulation_str: &str) -> Result<(), String> {
        let (reg, _, _, tar) = Regulation::try_from_string(regulation_str)?;
        let regulator = VarId::new(reg.as_str())?;
        let target = VarId::new(tar.as_str())?;
        self.remove_regulation(&regulator, &target)
    }

    /// **(internal)** Remove all `Regulations` where `variable` figures (as either regulator or
    /// target) from this `RegulationsState`.
    /// Returns `Err` when the variable is invalid.
    fn remove_all_regulations_var(&mut self, variable: &VarId) -> Result<(), String> {
        self.assert_valid_variable(variable)?;
        self.regulations
            .retain(|r| r.regulator != *variable && r.target != *variable);
        Ok(())
    }
}

/// Several utility methods to manipulate with layouts.
impl RegulationsState {
    /// Add a new empty `Layout` with given `layout_id` and `name` to this `RegulationsState`.
    ///
    /// Returns `Err` if `layout_id` is already being used for some other `Layout` in
    /// this `RegulationsState`.
    fn add_empty_layout(&mut self, layout_id: LayoutId, name: &str) -> Result<(), String> {
        self.assert_no_layout(&layout_id)?;
        self.layouts.insert(layout_id, Layout::new(name));
        Ok(())
    }

    /// Add a new `Layout` with given `layout_id` and `name` to this `RegulationsState`. The layout
    /// will contain nodes for same variables as layout `template_layout_id`, but all of them at a
    /// default position.
    ///
    /// Returns `Err` if `layout_id` is already being used for some other `Layout` in
    /// this `RegulationsState`, or if `template_layout_id` does not exist.
    pub fn add_layout_from_vars(
        &mut self,
        layout_id: LayoutId,
        name: &str,
        template_layout_id: &LayoutId,
    ) -> Result<(), String> {
        self.assert_no_layout(&layout_id)?;
        self.assert_valid_layout(template_layout_id)?;

        let template_layout = self.get_layout(template_layout_id)?;
        let layout = Layout::new_from_another_default(name, template_layout);
        self.layouts.insert(layout_id, layout);
        Ok(())
    }

    /// Add a new `Layout` with given `layout_id` and `name` to this `RegulationsState`. The layout
    /// will be a direct copy of another existing layout given by id `template_layout_id`.
    ///
    /// Returns `Err` if `layout_id` is already being used for some other `Layout` in
    /// this `RegulationsState`, or if `template_layout_id` does not exist.
    pub fn add_layout_copy(
        &mut self,
        layout_id: LayoutId,
        name: &str,
        template_layout_id: &LayoutId,
    ) -> Result<(), String> {
        self.assert_no_layout(&layout_id)?;
        self.assert_valid_layout(template_layout_id)?;

        let template_layout = self.get_layout(template_layout_id)?;
        let layout = Layout::new_from_another_copy(name, template_layout);
        self.layouts.insert(layout_id, layout);
        Ok(())
    }

    /// Remove a `Layout` with given `layout_id` from this `RegulationsState`. Default layout
    /// can not be deleted.
    ///
    /// Returns `Err` in case the `id` is not a valid identifier in this `RegulationsState`.
    fn remove_layout(&mut self, layout_id: &LayoutId) -> Result<(), String> {
        self.assert_valid_layout(layout_id)?;
        if *layout_id == RegulationsState::get_default_layout_id() {
            return Err("Default layout can not be deleted.".to_string());
        }
        if self.layouts.remove(layout_id).is_none() {
            panic!("Error when removing layout {layout_id} from the layout map.")
        }
        Ok(())
    }

    /// Remove a `Layout` with given `id` from this `RegulationsState`.
    ///
    /// Returns `Err` in case the `id` is not a valid identifier in this `RegulationsState`.
    pub fn remove_layout_by_str(&mut self, id: &str) -> Result<(), String> {
        let layout_id = LayoutId::new(id)?;
        self.remove_layout(&layout_id)
    }

    /// Update position of a node for variable `var_id` in layout `layout_id`.
    ///
    /// Returns `Err` in case one of the ids is not a valid for this `RegulationsState`.
    fn update_node_position(
        &mut self,
        layout_id: &LayoutId,
        var_id: &VarId,
        px: f32,
        py: f32,
    ) -> Result<(), String> {
        self.assert_valid_layout(layout_id)?;
        self.assert_valid_variable(var_id)?;

        self.layouts
            .get_mut(layout_id)
            .ok_or(format!("Error accessing layout {layout_id} in layout map"))?
            .update_node_position(var_id, px, py)
    }

    /// **(internal)** Utility method to add a variable node to a given layout.
    /// The node is inserted to a default position x=0,y=0.
    fn insert_to_layout(&mut self, var_id: VarId, layout_id: &LayoutId) -> Result<(), String> {
        self.assert_valid_variable(&var_id)?;
        self.assert_valid_layout(layout_id)?;

        let layout = self.layouts.get_mut(layout_id).unwrap();
        layout.add_default_node(var_id)?;
        Ok(())
    }

    /// **(internal)** Shorthand method for adding a variable node to a default layout.
    /// The node is inserted to a default position x=0,y=0.
    fn insert_to_default_layout(&mut self, var_id: VarId) -> Result<(), String> {
        let default_layout_id = RegulationsState::get_default_layout_id();
        self.insert_to_layout(var_id, &default_layout_id)
    }

    /// **(internal)** Shorthand method for adding a variable node to all layouts.
    /// The node is always inserted to a default position x=0,y=0.
    fn insert_to_all_layouts(&mut self, var_id: VarId) -> Result<(), String> {
        for layout in self.layouts.values_mut() {
            layout.add_default_node(var_id.clone())?;
        }
        Ok(())
    }

    /// **(internal)** Utility method to remove a variable node from a given layout.
    fn remove_from_layout(&mut self, var_id: &VarId, layout_id: &LayoutId) -> Result<(), String> {
        self.assert_valid_variable(var_id)?;
        self.assert_valid_layout(layout_id)?;

        let layout = self.layouts.get_mut(layout_id).unwrap();
        layout.remove_node(var_id)
    }

    /// **(internal)** Utility method to remove a variable node from a default layout.
    fn remove_from_default_layout(&mut self, var_id: &VarId) -> Result<(), String> {
        let default_layout_id = RegulationsState::get_default_layout_id();
        self.remove_from_layout(var_id, &default_layout_id)
    }

    /// **(internal)** Utility method to remove a variable node from all layouts.
    fn remove_from_all_layouts(&mut self, var_id: &VarId) -> Result<(), String> {
        for layout in self.layouts.values_mut() {
            layout.remove_node(var_id)?
        }
        Ok(())
    }
}

/// Several utility methods to assert (non-)existence of variables/regulations/layouts in the
/// current state.
impl RegulationsState {
    /// **(internal)** Utility method to ensure there is no regulation between the two variables yet.
    fn assert_no_regulation(&self, regulator: &VarId, target: &VarId) -> Result<(), String> {
        if self.get_regulation(regulator, target).is_err() {
            Ok(())
        } else {
            Err(format!(
                "Invalid regulation: {regulator} already regulates {target}."
            ))
        }
    }

    /// **(internal)** Utility method to ensure there is a regulation between the two variables.
    fn assert_valid_regulation(&self, regulator: &VarId, target: &VarId) -> Result<(), String> {
        if self.get_regulation(regulator, target).is_err() {
            Err(format!(
                "Invalid regulation: regulation between {regulator} and {target} does not exist."
            ))
        } else {
            Ok(())
        }
    }

    /// **(internal)** Utility method to ensure there is no variable with given Id yet.
    fn assert_no_variable(&self, var_id: &VarId) -> Result<(), String> {
        if self.get_var_name(var_id).is_err() {
            Ok(())
        } else {
            Err(format!(
                "Invalid variable: Variable with id {var_id} already exists."
            ))
        }
    }

    /// **(internal)** Utility method to ensure there is a variable with given Id.
    fn assert_valid_variable(&self, var_id: &VarId) -> Result<(), String> {
        if self.get_var_name(var_id).is_err() {
            Err(format!(
                "Invalid variable: Variable with id {var_id} does not exist."
            ))
        } else {
            Ok(())
        }
    }

    /// **(internal)** Utility method to ensure there is no layout with given Id yet.
    fn assert_no_layout(&self, layout_id: &LayoutId) -> Result<(), String> {
        if self.get_layout(layout_id).is_err() {
            Ok(())
        } else {
            Err(format!(
                "Invalid layout: Layout with id {layout_id} already exists."
            ))
        }
    }

    /// **(internal)** Utility method to ensure there is a layout with given Id.
    fn assert_valid_layout(&self, layout_id: &LayoutId) -> Result<(), String> {
        if self.get_layout(layout_id).is_err() {
            Err(format!(
                "Invalid layout: Layout with id {layout_id} does not exist."
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::layout::{LayoutId, NodePosition};
    use crate::sketchbook::RegulationsState;

    /// Test manually creating `RegulationsState` and mutating it by adding/removing variables
    /// or regulations.
    #[test]
    fn test_manually_editing() {
        let mut reg_state = RegulationsState::new();
        assert_eq!(reg_state.num_vars(), 0);

        // add variables a, b, c
        reg_state.add_var_by_str("a", "a_name").unwrap();
        reg_state.add_var_by_str("b", "b_name").unwrap();
        // same names should not be an issue
        reg_state.add_var_by_str("c", "a_name").unwrap();
        assert_eq!(reg_state.num_vars(), 3);
        // adding same ID again should cause error
        assert!(reg_state.add_var_by_str("a", "a_name").is_err());

        // add regulations a -> a, a -> b, b -> c, c -> a
        reg_state.add_regulation_by_str("a -> a").unwrap();
        reg_state.add_regulation_by_str("a -> b").unwrap();
        reg_state.add_regulation_by_str("b -> c").unwrap();
        reg_state.add_regulation_by_str("c -> a").unwrap();
        // adding same reg again or with invalid var should cause error
        assert!(reg_state.add_regulation_by_str("a -> a").is_err());
        assert!(reg_state.add_regulation_by_str("a -| a").is_err());
        assert!(reg_state.add_regulation_by_str("X -> a").is_err());
        assert!(reg_state.add_regulation_by_str("a -@ a").is_err());
        assert_eq!(reg_state.num_regulations(), 4);

        // rename variable
        let var_a = reg_state.get_var_id("a").unwrap();
        reg_state.set_var_name(&var_a, "new_name").unwrap();
        assert_eq!(reg_state.get_var_name(&var_a).unwrap(), "new_name");

        // remove variable and check that all its regulations disappear, try re-adding it then
        reg_state.remove_var(&var_a).unwrap();
        assert!(reg_state.get_var_name(&var_a).is_err());
        let var_b = reg_state.get_var_id("b").unwrap();
        assert!(reg_state.get_regulation(&var_a, &var_b).is_err());
        assert_eq!(reg_state.num_regulations(), 1);
        assert!(reg_state.add_var(var_a, "name_a").is_ok());

        // test removing a regulation and then re-adding it again
        reg_state.remove_regulation_by_str("b -> c").unwrap();
        let var_c = reg_state.get_var_id("c").unwrap();
        assert!(reg_state.get_regulation(&var_b, &var_c).is_err());
        assert!(reg_state.add_regulation_by_str("b -> c").is_ok());
        assert_eq!(reg_state.num_regulations(), 1);
    }

    #[test]
    fn test_new_from_vars() {
        let var_id_name_pairs = vec![("a_id", "a_name"), ("a_id", "b_name")];
        assert!(RegulationsState::new_from_vars(var_id_name_pairs).is_err());

        let var_id_name_pairs = vec![("a_id", "a_name"), ("b_id", "b_name")];
        let reg_state = RegulationsState::new_from_vars(var_id_name_pairs).unwrap();
        assert_eq!(reg_state.num_vars(), 2);
        assert_eq!(reg_state.num_regulations(), 0);
        assert!(reg_state.is_valid_var_id_str("a_id"));
        assert!(reg_state.is_valid_var_id_str("b_id"));
        assert!(!reg_state.is_valid_var_id_str("c_id"));
    }

    #[test]
    fn test_layout_manipulation() {
        let var_id_name_pairs = vec![("a_id", "a_name"), ("b_id", "b_name")];
        let mut reg_state = RegulationsState::new_from_vars(var_id_name_pairs).unwrap();
        assert_eq!(reg_state.num_layouts(), 1);

        // check default layout
        let default_layout_id = RegulationsState::get_default_layout_id();
        let default_layout_name = RegulationsState::get_default_layout_name();
        let default_layout = reg_state.get_layout(&default_layout_id).unwrap();
        assert!(reg_state.is_valid_layout_id(&default_layout_id));
        assert_eq!(default_layout.get_num_nodes(), 2);
        assert_eq!(default_layout.get_layout_name(), default_layout_name);

        // change default layout's nodes
        let var_id = reg_state.get_var_id("a_id").unwrap();
        reg_state
            .update_node_position(&default_layout_id, &var_id, 2., 2.)
            .unwrap();
        let position = reg_state
            .get_node_position(&default_layout_id, &var_id)
            .unwrap();
        assert_eq!(position, &NodePosition(2., 2.));

        // add layouts (one as vars with default nodes, and other as direct copy)
        let new_id_1 = reg_state.generate_layout_id("new_layout");
        reg_state
            .add_layout_from_vars(new_id_1.clone(), "new_layout", &default_layout_id)
            .unwrap();
        let position = reg_state.get_node_position(&new_id_1, &var_id).unwrap();
        assert_eq!(position, &NodePosition(0., 0.));

        let new_id_2 = reg_state.generate_layout_id("another_layout");
        reg_state
            .add_layout_copy(new_id_2.clone(), "new_layout", &default_layout_id)
            .unwrap();
        let position = reg_state.get_node_position(&new_id_2, &var_id).unwrap();
        assert_eq!(position, &NodePosition(2., 2.));
    }
}
