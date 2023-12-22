use crate::sketchbook::layout::NodePosition;
use crate::sketchbook::{
    Layout, LayoutId, LayoutIterator, ModelState, Regulation, RegulationIterator, VarId, Variable,
    VariableIterator,
};
use std::str::FromStr;

/// Id (and name) of the initial default layout.
const DEFAULT_LAYOUT_ID: &str = "default_layout";

/// Some basic utility methods for inspecting the `ModelState`.
impl ModelState {
    /// The number of variables in this `ModelState`.
    pub fn num_vars(&self) -> usize {
        self.variables.len()
    }

    /// The number of layouts in this `ModelState`.
    pub fn num_layouts(&self) -> usize {
        self.layouts.len()
    }

    /// The number of regulations in this `ModelState`.
    pub fn num_regulations(&self) -> usize {
        self.regulations.len()
    }

    /// Check if there is a variable with given Id.
    pub fn is_valid_var_id(&self, var_id: &VarId) -> bool {
        self.variables.contains_key(var_id)
    }

    /// Check if the given `id` corresponds to some variable's valid Id.
    pub fn is_valid_var_id_str(&self, id: &str) -> bool {
        if let Ok(var_id) = VarId::from_str(id) {
            self.is_valid_var_id(&var_id)
        } else {
            false
        }
    }

    /// Check if there is a layout with given Id.
    pub fn is_valid_layout_id(&self, layout_id: &LayoutId) -> bool {
        self.layouts.contains_key(layout_id)
    }

    /// Check if the given `id` corresponds to some layout's valid Id.
    pub fn is_valid_layout_id_str(&self, id: &str) -> bool {
        if let Ok(layout_id) = LayoutId::from_str(id) {
            self.is_valid_layout_id(&layout_id)
        } else {
            false
        }
    }

    /// Return a valid variable's `VarId` corresponding to the Id given by a `String`.
    ///
    /// Return `Err` if such variable does not exist (and the ID is invalid).
    pub fn get_var_id(&self, id: &str) -> Result<VarId, String> {
        let var_id = VarId::from_str(id)?;
        if self.is_valid_var_id(&var_id) {
            return Ok(var_id);
        }
        Err(format!("Variable with ID {id} does not exist."))
    }

    /// Return a `Variable` corresponding to a given `VarId`.
    ///
    /// Return `Err` if such variable does not exist (the ID is invalid in this context).
    pub fn get_variable(&self, var_id: &VarId) -> Result<&Variable, String> {
        let variable = self
            .variables
            .get(var_id)
            .ok_or(format!("Variable with ID {var_id} does not exist."))?;
        Ok(variable)
    }

    /// Shortcut to return a name of the variable corresponding to a given `VarId`.
    ///
    /// Return `Err` if such variable does not exist (the ID is invalid in this context).
    pub fn get_var_name(&self, var_id: &VarId) -> Result<&str, String> {
        let variable = self
            .variables
            .get(var_id)
            .ok_or(format!("Variable with ID {var_id} does not exist."))?;
        Ok(variable.get_name())
    }

    /// Find a `Regulation` between two variables if it exists.
    ///
    /// Return `Err` if one of variable ids is invalid or the regulation does not exist.
    pub fn get_regulation(&self, regulator: &VarId, target: &VarId) -> Result<&Regulation, String> {
        if !self.is_valid_var_id(regulator) {
            return Err(format!(
                "Regulator variable with ID {regulator} does not exist."
            ));
        }
        if !self.is_valid_var_id(target) {
            return Err(format!("Target variable with ID {target} does not exist."));
        }
        self.regulations
            .iter()
            .find(|r| r.get_regulator() == regulator && r.get_target() == target)
            .ok_or(format!(
                "Regulation between {regulator} and {target} does not exist."
            ))
    }

    /// Return a `Layout` corresponding to the given `LayoutId`.
    ///
    /// Return `Err` if the `LayoutId` is invalid.
    pub fn get_layout(&self, id: &LayoutId) -> Result<&Layout, String> {
        self.layouts
            .get(id)
            .ok_or(format!("Layout with ID {id} does not exist."))
    }

    /// Return a valid layout's `LayoutId` corresponding to the Id given by a `String`.
    ///
    /// Return `Err` if such variable does not exist (and the ID is invalid).
    pub fn get_layout_id(&self, id: &str) -> Result<LayoutId, String> {
        let layout_id = LayoutId::from_str(id)?;
        if self.is_valid_layout_id(&layout_id) {
            return Ok(layout_id);
        }
        Err(format!("Layout with ID {id} does not exist."))
    }

    /// Shorthand for getting a string name of a layout.
    pub fn get_layout_name(&self, id: &LayoutId) -> Result<&String, String> {
        Ok(self.get_layout(id)?.get_layout_name())
    }

    /// Shorthand for getting a position of a node for given variable in a given layout.
    pub fn get_node_position(
        &self,
        layout_id: &LayoutId,
        var_id: &VarId,
    ) -> Result<&NodePosition, String> {
        self.get_layout(layout_id)?.get_node_position(var_id)
    }

    /// Return a sorted list of variables that regulate the given `target` variable.
    pub fn regulators(&self, target: &VarId) -> Result<Vec<&VarId>, String> {
        if !self.is_valid_var_id(target) {
            return Err(format!("Target variable with ID {target} does not exist."));
        }

        let mut regulators: Vec<&VarId> = self
            .regulations
            .iter()
            .filter(|r| r.get_target() == target)
            .map(|r| r.get_regulator())
            .collect();
        regulators.sort();
        Ok(regulators)
    }

    /// Return a sorted list of variables that are regulated by the given `regulator` variable.
    pub fn targets(&self, regulator: &VarId) -> Result<Vec<&VarId>, String> {
        if !self.is_valid_var_id(regulator) {
            return Err(format!(
                "Regulator variable with ID {regulator} does not exist."
            ));
        }

        let mut targets: Vec<&VarId> = self
            .regulations
            .iter()
            .filter(|r| r.get_regulator() == regulator)
            .map(|r| r.get_target())
            .collect();
        targets.sort();
        Ok(targets)
    }

    /// Return an iterator over all variables of this graph.
    pub fn variables(&self) -> VariableIterator {
        self.variables.keys()
    }

    /// Return an iterator over all regulations of this graph.
    pub fn regulations(&self) -> RegulationIterator {
        self.regulations.iter()
    }

    /// Return an iterator over all layouts of this graph.
    pub fn layouts(&self) -> LayoutIterator {
        self.layouts.keys()
    }

    /// Static fn to get `LayoutId` of the default layout (same for all `ModelStates`).
    pub fn get_default_layout_id() -> LayoutId {
        LayoutId::new(DEFAULT_LAYOUT_ID).unwrap()
    }

    /// Static fn to get name of the default layout (same for all `ModelStates`).
    pub fn get_default_layout_name() -> &'static str {
        DEFAULT_LAYOUT_ID
    }
}
