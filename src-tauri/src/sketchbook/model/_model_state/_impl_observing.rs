use crate::sketchbook::ids::{LayoutId, UninterpretedFnId, VarId};
use crate::sketchbook::layout::Layout;
use crate::sketchbook::layout::NodePosition;
use crate::sketchbook::model::{
    LayoutIterator, ModelState, Regulation, RegulationIterator, UninterpretedFn,
    UninterpretedFnIterator, UpdateFn, UpdateFnIterator, Variable, VariableIterator,
};
use std::collections::HashSet;

use std::str::FromStr;

/// Id (and also name) of the initial default layout.
const DEFAULT_LAYOUT_ID: &str = "default";

/// Some basic utility methods for inspecting the `ModelState`.
impl ModelState {
    /// The number of variables in this `ModelState`.
    pub fn num_vars(&self) -> usize {
        self.variables.len()
    }

    /// The number of uninterpreted_fns in this `ModelState`.
    pub fn num_uninterpreted_fns(&self) -> usize {
        self.uninterpreted_fns.len()
    }

    /// The number of layouts in this `ModelState`.
    pub fn num_layouts(&self) -> usize {
        self.layouts.len()
    }

    /// The number of regulations in this `ModelState`.
    pub fn num_regulations(&self) -> usize {
        self.regulations.len()
    }

    /// The number of placeholder variables in this `ModelState`.
    pub(crate) fn num_placeholder_vars(&self) -> usize {
        self.placeholder_variables.len()
    }

    /// Check if there is a variable with given Id.
    pub fn is_valid_var_id(&self, var_id: &VarId) -> bool {
        self.variables.contains_key(var_id)
    }

    /// Check if there is a placeholder variable with given Id.
    pub(crate) fn is_valid_placeholder_var_id(&self, var_id: &VarId) -> bool {
        self.placeholder_variables.contains(var_id)
    }

    /// Check if the given `id` corresponds to some variable's valid Id.
    pub fn is_valid_var_id_str(&self, id: &str) -> bool {
        if let Ok(var_id) = VarId::from_str(id) {
            self.is_valid_var_id(&var_id)
        } else {
            false
        }
    }

    /// Check if there is a uninterpreted fn with given Id.
    pub fn is_valid_uninterpreted_fn_id(&self, fn_id: &UninterpretedFnId) -> bool {
        self.uninterpreted_fns.contains_key(fn_id)
    }

    /// Check if the given `id` corresponds to some uninterpreted fn's valid Id.
    pub fn is_valid_uninterpreted_fn_id_str(&self, id: &str) -> bool {
        if let Ok(fn_id) = UninterpretedFnId::from_str(id) {
            self.is_valid_uninterpreted_fn_id(&fn_id)
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

    /// Return a valid variable's `VarId` corresponding to the given str `id`.
    ///
    /// Return `Err` if such variable does not exist (and the ID is invalid).
    pub fn get_var_id(&self, id: &str) -> Result<VarId, String> {
        let var_id = VarId::from_str(id)?;
        if self.is_valid_var_id(&var_id) {
            return Ok(var_id);
        }
        Err(format!(
            "Variable with ID {id} does not exist in this model."
        ))
    }

    /// Return a valid placeholder variable's `VarId` corresponding to the given str `id`.
    ///
    /// Return `Err` if such variable does not exist (and the ID is invalid).
    pub(crate) fn get_placeholder_var_id(&self, id: &str) -> Result<VarId, String> {
        let var_id = VarId::from_str(id)?;
        if self.is_valid_placeholder_var_id(&var_id) {
            return Ok(var_id);
        }
        Err(format!(
            "Placeholder variable with ID {id} does not exist in this model."
        ))
    }

    /// Return a `Variable` corresponding to a given `VarId`.
    ///
    /// Return `Err` if such variable does not exist (the ID is invalid in this context).
    pub fn get_variable(&self, var_id: &VarId) -> Result<&Variable, String> {
        let variable = self.variables.get(var_id).ok_or(format!(
            "Variable with ID {var_id} does not exist in this model."
        ))?;
        Ok(variable)
    }

    /// Return a valid uninterpreted fn's `UninterpretedFnId` corresponding to the given str `id`.
    ///
    /// Return `Err` if no such uninterpreted fn exists (and the ID is invalid).
    pub fn get_uninterpreted_fn_id(&self, id: &str) -> Result<UninterpretedFnId, String> {
        let fn_id = UninterpretedFnId::from_str(id)?;
        if self.is_valid_uninterpreted_fn_id(&fn_id) {
            return Ok(fn_id);
        }
        Err(format!(
            "UninterpretedFn with ID {id} does not exist in this model."
        ))
    }

    /// Return a `UninterpretedFn` corresponding to a given `UninterpretedFnId`.
    ///
    /// Return `Err` if no such uninterpreted fn exists (the ID is invalid in this context).
    pub fn get_uninterpreted_fn(
        &self,
        fn_id: &UninterpretedFnId,
    ) -> Result<&UninterpretedFn, String> {
        let uninterpreted_fn = self.uninterpreted_fns.get(fn_id).ok_or(format!(
            "UninterpretedFn with ID {fn_id} does not exist in this model."
        ))?;
        Ok(uninterpreted_fn)
    }

    /// Return arity of a `UninterpretedFn` corresponding to a given `UninterpretedFnId`.
    ///
    /// Return `Err` if no such uninterpreted fn exists (the ID is invalid in this context).
    pub fn get_uninterpreted_fn_arity(&self, fn_id: &UninterpretedFnId) -> Result<usize, String> {
        let uninterpreted_fn = self.uninterpreted_fns.get(fn_id).ok_or(format!(
            "UninterpretedFn with ID {fn_id} does not exist in this model."
        ))?;
        Ok(uninterpreted_fn.get_arity())
    }

    /// Shortcut to return a name of the variable corresponding to a given `VarId`.
    ///
    /// Return `Err` if such variable does not exist (the ID is invalid in this context).
    pub fn get_var_name(&self, var_id: &VarId) -> Result<&str, String> {
        let variable = self.variables.get(var_id).ok_or(format!(
            "Variable with ID {var_id} does not exist in this model."
        ))?;
        Ok(variable.get_name())
    }

    /// Find a `Regulation` between two variables if it exists.
    ///
    /// Return `Err` if one of variable ids is invalid or the regulation does not exist.
    pub fn get_regulation(&self, regulator: &VarId, target: &VarId) -> Result<&Regulation, String> {
        if !self.is_valid_var_id(regulator) {
            return Err(format!(
                "Regulator variable with ID {regulator} does not exist in this model."
            ));
        }
        if !self.is_valid_var_id(target) {
            return Err(format!(
                "Target variable with ID {target} does not exist in this model."
            ));
        }
        self.regulations
            .iter()
            .find(|r| r.get_regulator() == regulator && r.get_target() == target)
            .ok_or(format!(
                "Regulation between {regulator} and {target} does not exist in this model."
            ))
    }

    /// Shorthand to find a `Regulation` between two variables (specified by strings) if it exists.
    ///
    /// Return `Err` if one of variable ids is invalid or the regulation does not exist.
    pub fn get_regulation_by_str(&self, reg: &str, target: &str) -> Result<&Regulation, String> {
        let reg_id = self.get_var_id(reg)?;
        let target_id = self.get_var_id(target)?;
        self.get_regulation(&reg_id, &target_id)
    }

    /// Return a `Layout` corresponding to the given `LayoutId`.
    ///
    /// Return `Err` if the `LayoutId` is invalid.
    pub fn get_layout(&self, id: &LayoutId) -> Result<&Layout, String> {
        self.layouts
            .get(id)
            .ok_or(format!("Layout with ID {id} does not exist in this model."))
    }

    /// Return a valid layout's `LayoutId` corresponding to the Id given by a `String`.
    ///
    /// Return `Err` if such variable does not exist (and the ID is invalid).
    pub fn get_layout_id(&self, id: &str) -> Result<LayoutId, String> {
        let layout_id = LayoutId::from_str(id)?;
        if self.is_valid_layout_id(&layout_id) {
            return Ok(layout_id);
        }
        Err(format!("Layout with ID {id} does not exist in this model."))
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
            return Err(format!(
                "Target variable with ID {target} does not exist in this model."
            ));
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
                "Regulator variable with ID {regulator} does not exist in this model."
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

    /// Get an update function for the given variable.
    pub fn get_update_fn(&self, var_id: &VarId) -> Result<&UpdateFn, String> {
        let update_fn = self.update_fns.get(var_id).ok_or(format!(
            "Variable with ID {var_id} does not exist in this model."
        ))?;
        Ok(update_fn)
    }

    /// Get an update function's expression for the given variable.
    pub fn get_update_fn_string(&self, var_id: &VarId) -> Result<&str, String> {
        let update_fn = self.update_fns.get(var_id).ok_or(format!(
            "Variable with ID {var_id} does not exist in this model."
        ))?;
        Ok(update_fn.get_fn_expression())
    }

    /// Get a list of variables with "empty" update function.
    /// Returned list contains string ID of each such variable
    pub fn get_vars_with_empty_update(&self) -> Vec<&str> {
        self.update_fns()
            .filter(|(_, update_fn)| update_fn.is_unspecified())
            .map(|(var_id, _)| var_id.as_str())
            .collect()
    }

    /// Check whether variable is used in any update function's expressions.
    /// This is important in case we want to safely delete it.
    ///
    /// We expect valid var id here.
    pub fn is_var_contained_in_updates(&self, var_id: &VarId) -> bool {
        // check that variable can be safely deleted (not contained in any update fn)
        let mut vars_in_update_fns = HashSet::new();
        for update_fn in self.update_fns.values() {
            let tmp_var_set = update_fn.collect_variables();
            vars_in_update_fns.extend(tmp_var_set);
        }
        vars_in_update_fns.contains(var_id)
    }

    /// Return an iterator over all variables (with IDs) of this model.
    pub fn variables(&self) -> VariableIterator {
        self.variables.iter()
    }

    /// Return an iterator over all update functions (with corresponding var IDs) of
    /// this model.
    pub fn update_fns(&self) -> UpdateFnIterator {
        self.update_fns.iter()
    }

    /// Return an iterator over all uninterpreted_fns (with IDs) of this model.
    pub fn uninterpreted_fns(&self) -> UninterpretedFnIterator {
        self.uninterpreted_fns.iter()
    }

    /// Return an iterator over all regulations of this model.
    pub fn regulations(&self) -> RegulationIterator {
        self.regulations.iter()
    }

    /// Return an iterator over all layouts (with IDs) of this model.
    pub fn layouts(&self) -> LayoutIterator {
        self.layouts.iter()
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
