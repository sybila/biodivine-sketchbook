use crate::sketchbook::ids::{PerturbationId, VarId};
use crate::sketchbook::perturbations::{Perturbation, PerturbationIterator, PerturbationManager};
use std::collections::HashMap;
use std::str::FromStr;

/// Creating new instances of `PerturbationManager`.
impl PerturbationManager {
    /// Instantiate `PerturbationManager` with empty sets of perturbations.
    pub fn new_empty() -> PerturbationManager {
        PerturbationManager {
            perturbations: HashMap::new(),
        }
    }

    /// Instantiate `PerturbationManager` with perturbations given as a list of ID-perturbation pairs.
    pub fn new_from_perturbations(
        perturbations: Vec<(&str, Perturbation)>,
    ) -> Result<PerturbationManager, String> {
        let mut perturbations_map = HashMap::with_capacity(perturbations.len());
        for (id, pert) in perturbations {
            let id = PerturbationId::new(id)?;
            if perturbations_map.insert(id.clone(), pert).is_some() {
                return Err(format!(
                    "Perturbation with id {id} already exists (id must be unique)."
                ));
            }
        }

        Ok(PerturbationManager {
            perturbations: perturbations_map,
        })
    }
}

/// Editing `PerturbationManager`.
impl PerturbationManager {
    /// Add a perturbation with the given ID and content.
    pub fn add_perturbation(
        &mut self,
        id: PerturbationId,
        perturbation: Perturbation,
    ) -> Result<(), String> {
        self.assert_no_perturbation(&id)?;
        self.perturbations.insert(id, perturbation);
        Ok(())
    }

    /// Add a perturbation with the given ID as a string.
    pub fn add_perturbation_by_str(
        &mut self,
        id: &str,
        perturbation: Perturbation,
    ) -> Result<(), String> {
        let id = PerturbationId::new(id)?;
        self.add_perturbation(id, perturbation)
    }

    /// Swap content of a perturbation with given `id`. The ID must be valid.
    /// This allows updating a perturbation while keeping its ID.
    pub fn swap_perturbation_content(
        &mut self,
        id: &PerturbationId,
        new_content: Perturbation,
    ) -> Result<(), String> {
        self.assert_valid_perturbation(id)?;
        self.perturbations.insert(id.clone(), new_content);
        Ok(())
    }

    /// Swap content of a perturbation with given `id` as a string.
    pub fn swap_perturbation_content_by_str(
        &mut self,
        id: &str,
        new_content: Perturbation,
    ) -> Result<(), String> {
        let pert_id = PerturbationId::new(id)?;
        self.swap_perturbation_content(&pert_id, new_content)
    }

    /// Change the ID of a perturbation.
    pub fn set_perturbation_id(
        &mut self,
        original_id: &PerturbationId,
        new_id: PerturbationId,
    ) -> Result<(), String> {
        self.assert_valid_perturbation(original_id)?;
        self.assert_no_perturbation(&new_id)?;

        if let Some(perturbation) = self.perturbations.remove(original_id) {
            self.perturbations.insert(new_id.clone(), perturbation);
        } else {
            panic!("Error when modifying perturbation's id in the perturbation map.");
        }
        Ok(())
    }

    /// Change the ID of a perturbation, with IDs given as string slices.
    pub fn set_perturbation_id_by_str(
        &mut self,
        original_id: &str,
        new_id: &str,
    ) -> Result<(), String> {
        let original_id = PerturbationId::new(original_id)?;
        let new_id = PerturbationId::new(new_id)?;
        self.set_perturbation_id(&original_id, new_id)
    }

    /// Remove a perturbation.
    pub fn remove_perturbation(&mut self, id: &PerturbationId) -> Result<(), String> {
        self.assert_valid_perturbation(id)?;
        self.perturbations.remove(id).unwrap();
        Ok(())
    }
}

/// Internal assertion utilities.
impl PerturbationManager {
    /// **(internal)** Utility method to ensure there is no perturbation with given ID yet.
    fn assert_no_perturbation(&self, id: &PerturbationId) -> Result<(), String> {
        if self.is_valid_perturbation_id(id) {
            Err(format!("Perturbation with id {id} already exists."))
        } else {
            Ok(())
        }
    }

    /// **(internal)** Utility method to ensure there is a perturbation with given ID.
    fn assert_valid_perturbation(&self, id: &PerturbationId) -> Result<(), String> {
        if self.is_valid_perturbation_id(id) {
            Ok(())
        } else {
            Err(format!("Perturbation with id {id} does not exist."))
        }
    }
}

/// Observing the `PerturbationManager`.
impl PerturbationManager {
    /// The number of perturbations in this `PerturbationManager`.
    pub fn num_perturbations(&self) -> usize {
        self.perturbations.len()
    }

    /// Check if there is a perturbation with given ID.
    pub fn is_valid_perturbation_id(&self, id: &PerturbationId) -> bool {
        self.perturbations.contains_key(id)
    }

    /// Return an iterator over all perturbations.
    pub fn perturbations_iter(&self) -> PerturbationIterator<'_> {
        self.perturbations.iter()
    }

    /// Return a valid `PerturbationId` corresponding to the given str `id`.
    ///
    /// Return `Err` if such perturbation does not exist (and the ID is invalid).
    pub fn get_perturbation_id(&self, id: &str) -> Result<PerturbationId, String> {
        let perturbation_id = PerturbationId::from_str(id)?;
        if self.is_valid_perturbation_id(&perturbation_id) {
            return Ok(perturbation_id);
        }
        Err(format!("Perturbation with ID {id} does not exist."))
    }

    /// Return a `Perturbation` corresponding to a given `PerturbationId`.
    ///
    /// Return `Err` if such perturbation does not exist (the ID is invalid in this context).
    pub fn get_perturbation(&self, id: &PerturbationId) -> Result<&Perturbation, String> {
        let perturbation = self
            .perturbations
            .get(id)
            .ok_or(format!("Perturbation with ID {id} does not exist."))?;
        Ok(perturbation)
    }

    /// Return IDs of all perturbations that list `var_id` among their perturbed variables.
    pub fn perturbations_containing_var(&self, var_id: &VarId) -> Vec<PerturbationId> {
        let mut pert_ids: Vec<PerturbationId> = self
            .perturbations
            .iter()
            .filter(|(_, perturb)| perturb.get_perturbed_vars().contains_key(var_id))
            .map(|(pert_id, _)| pert_id.clone())
            .collect();
        pert_ids.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        pert_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn perturbations_containing_var() {
        let var_a = VarId::new("A").unwrap();
        let var_b = VarId::new("B").unwrap();
        let mut perturbed_vars = BTreeMap::new();
        perturbed_vars.insert(var_a.clone(), true);
        let pert_with_a = Perturbation::new("pert_a", perturbed_vars);
        let pert_empty = Perturbation::new_empty("pert_empty");

        let manager = PerturbationManager::new_from_perturbations(vec![
            ("pert_a", pert_with_a),
            ("pert_empty", pert_empty),
        ])
        .unwrap();

        assert_eq!(
            manager
                .perturbations_containing_var(&var_a)
                .iter()
                .map(|id| id.as_str())
                .collect::<Vec<_>>(),
            vec!["pert_a"]
        );
        assert!(manager.perturbations_containing_var(&var_b).is_empty());
    }
}
