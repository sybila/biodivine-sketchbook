use crate::sketchbook::ids::VarId;
use std::collections::BTreeMap;

/// A typesafe representation wrapping various kinds of perturbations.
/// Each perturbation has a `name` field and perturbation map <varID> -> <value>.
/// Variables not in the map are considered unperturbed.
/// It can also be annotated using a string `annotation` field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Perturbation {
    name: String,
    annotation: String,
    perturbed_vars: BTreeMap<VarId, bool>,
}

/// Creating perturbations.
impl Perturbation {
    /// Create a new perturbation with a given name and empty set of perturbed variables.
    /// Annotation is left empty initially.
    pub fn new_empty(name: &str) -> Perturbation {
        Perturbation {
            name: name.to_string(),
            annotation: String::new(),
            perturbed_vars: BTreeMap::new(),
        }
    }

    /// Create a new perturbation given a name and a map of perturbed variables.
    /// Annotation is left empty initially.
    pub fn new(name: &str, perturbed_vars: BTreeMap<VarId, bool>) -> Perturbation {
        Perturbation {
            name: name.to_string(),
            annotation: String::new(),
            perturbed_vars,
        }
    }

    /// Update the `annotation` property.
    pub fn with_annotation(mut self, annotation: &str) -> Self {
        self.annotation = annotation.to_string();
        self
    }
}

/// Observing and editing perturbations.
impl Perturbation {
    /// Get the name of this perturbation.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get the annotation of this perturbation.
    pub fn get_annotation(&self) -> &str {
        &self.annotation
    }

    /// Get reference to the perturbed variables map.
    pub fn get_perturbed_vars(&self) -> &BTreeMap<VarId, bool> {
        &self.perturbed_vars
    }

    /// Get mutable reference to the perturbed variables map.
    pub fn get_perturbed_vars_mut(&mut self) -> &mut BTreeMap<VarId, bool> {
        &mut self.perturbed_vars
    }

    /// Update the name of this perturbation.
    pub fn set_name(&mut self, name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Perturbation name cannot be empty.".to_string());
        }
        self.name = name.to_string();
        Ok(())
    }

    /// Update the annotation of this perturbation.
    pub fn set_annotation(&mut self, annotation: &str) {
        self.annotation = annotation.to_string();
    }

    /// Set the full map of perturbed variables, replacing the original.
    pub fn set_perturbed_vars(&mut self, new_perturbed_vars: BTreeMap<VarId, bool>) {
        self.perturbed_vars = new_perturbed_vars;
    }

    /// Update ID of one of the perturbed variables if present in the map, otherwise do nothing.
    pub fn set_var_id_if_present(&mut self, original_id: &VarId, new_id: VarId) {
        self.perturbed_vars
            .remove(original_id)
            .map(|value| self.perturbed_vars.insert(new_id.clone(), value));
    }

    /// Update ID of one of the perturbed variables if present in the map, otherwise do nothing.
    pub fn set_var_id_by_str_if_present(
        &mut self,
        original_id: &str,
        new_id: &str,
    ) -> Result<(), String> {
        let original_id = VarId::new(original_id)?;
        let new_id = VarId::new(new_id)?;
        self.set_var_id_if_present(&original_id, new_id);
        Ok(())
    }

    /// Set (or update) value of the perturbed variable.
    pub fn set_var_value(&mut self, var_id: &VarId, value: bool) {
        self.perturbed_vars.insert(var_id.clone(), value);
    }

    /// Set (or update) value of the perturbed variable.
    pub fn set_var_value_by_str(&mut self, var_id: &str, value: bool) -> Result<(), String> {
        let var_id = VarId::new(var_id)?;
        self.set_var_value(&var_id, value);
        Ok(())
    }
}
