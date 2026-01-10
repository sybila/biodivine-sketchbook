use crate::sketchbook::data_structs::SketchData;
use crate::sketchbook::ids::VarId;
use crate::sketchbook::model::ModelState;
use crate::sketchbook::observations::{Dataset, ObservationManager};
use crate::sketchbook::properties::{DynProperty, PropertyManager, StatProperty};
use crate::sketchbook::Sketch;

use biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext;
use std::collections::HashSet;

/// Utility functions for creating or modifying sketch instances.
impl Sketch {
    /// Parse and validate all components of `Sketch` from a corresponding `SketchData` instance.
    pub fn components_from_sketch_data(
        sketch_data: &SketchData,
    ) -> Result<(ModelState, ObservationManager, PropertyManager), String> {
        let datasets = sketch_data
            .datasets
            .iter()
            .map(|d| d.to_dataset())
            .collect::<Result<Vec<Dataset>, String>>()?;
        let dyn_properties = sketch_data
            .dyn_properties
            .iter()
            .map(|prop_data| prop_data.to_property())
            .collect::<Result<Vec<DynProperty>, String>>()?;
        let stat_properties = sketch_data
            .stat_properties
            .iter()
            .map(|prop_data| prop_data.to_property())
            .collect::<Result<Vec<StatProperty>, String>>()?;

        let model = ModelState::new_from_model_data(&sketch_data.model)?;
        let obs_manager = ObservationManager::from_datasets(
            sketch_data
                .datasets
                .iter()
                .map(|d| d.id.as_str())
                .zip(datasets)
                .collect(),
        )?;
        let prop_manager = PropertyManager::new_from_properties(
            sketch_data
                .dyn_properties
                .iter()
                .map(|d| d.id.as_str())
                .zip(dyn_properties)
                .collect(),
            sketch_data
                .stat_properties
                .iter()
                .map(|d| d.id.as_str())
                .zip(stat_properties)
                .collect(),
        )?;
        Ok((model, obs_manager, prop_manager))
    }

    /// Create a new `Sketch` instance given a corresponding `SketchData` object.
    pub fn new_from_sketch_data(sketch_data: &SketchData) -> Result<Sketch, String> {
        let (model, obs_manager, prop_manager) = Self::components_from_sketch_data(sketch_data)?;
        Ok(Sketch {
            model,
            observations: obs_manager,
            properties: prop_manager,
            annotation: sketch_data.annotation.clone(),
        })
    }

    /// Modify this `Sketch` instance by loading all its components from a corresponding
    /// `SketchData` instance. The original sketch information is forgotten.
    pub fn modify_from_sketch_data(&mut self, sketch_data: &SketchData) -> Result<(), String> {
        let (model, obs_manager, prop_manager) = Self::components_from_sketch_data(sketch_data)?;
        self.model = model;
        self.observations = obs_manager;
        self.properties = prop_manager;
        Ok(())
    }

    /// Modify this `Sketch` instance by loading all its components from a different
    /// `Sketch` instance. The original sketch information is forgotten.
    pub fn modify_from_sketch(&mut self, other_sketch: &Sketch) {
        *self = other_sketch.clone();
    }

    /// Modify this `Sketch` instance to a default (empty) settings.
    pub fn set_to_empty(&mut self) {
        self.model = ModelState::default();
        self.observations = ObservationManager::default();
        self.properties = PropertyManager::default();
        self.annotation = String::new();
    }

    /// Get a copy of this sketch with pre-processed datasets so that they match the
    /// model's variables.
    ///
    /// Any extra variables (not present in the model) are removed from all datasets,
    /// and any missing variables (present in the model but not in the dataset) are
    /// added with unspecified values (at the end of the dataset's variable list).
    pub fn with_processed_datasets(&self) -> Sketch {
        let mut modified_sketch = self.clone();

        let model_variables: HashSet<&VarId> =
            self.model.variables().map(|(var_id, _)| var_id).collect();
        for (dataset_id, dataset) in self.observations.datasets() {
            // Check if the variable sets match exactly first (if so, nothing to do)
            let dataset_variables: HashSet<&VarId> = HashSet::from_iter(dataset.variables().iter());
            if model_variables == dataset_variables {
                continue;
            }

            let mut modified_dataset = dataset.clone();

            // Remove extra variables from the dataset
            let extra_variables = dataset_variables.difference(&model_variables);
            for &var_id in extra_variables {
                // Can safely unwrap, the variable ID must be valid in the dataset
                modified_dataset.remove_var(var_id).unwrap();
            }

            // Add missing variables to the dataset (in alphabetical order for determinism)
            let mut missing_variables = model_variables
                .difference(&dataset_variables)
                .collect::<Vec<_>>();
            missing_variables.sort();
            for &var_id in missing_variables {
                // Place the new variable at the end of the var list
                let new_var_idx = modified_dataset.num_variables();
                // Can safely unwrap, the variable ID is not yet present in the dataset
                modified_dataset
                    .add_var_default(var_id.clone(), new_var_idx)
                    .unwrap();
            }

            // And finally, swap the original dataset content for the modified one
            // Can safely unwrap, the dataset ID must be valid in the sketch
            modified_sketch
                .observations
                .swap_dataset_content(dataset_id, modified_dataset)
                .unwrap();
        }

        modified_sketch
    }

    /// Get annotation string.
    pub fn get_annotation(&self) -> &str {
        &self.annotation
    }

    /// Get number of BN "parameters", e.g., number of symbolic variables needed to encode
    /// the uninterprete functions. The number of interpretations should be 2^{num_parameters}.
    ///
    /// All unused function symbols are pruned first, and function expressions are substituted
    /// in before computing the number of params.
    pub fn get_num_parameters(&self) -> usize {
        let bn = self.model.to_bn_with_plain_regulations();
        // remove all unused function symbols, as these would cause problems later
        let bn = bn.prune_unused_parameters();
        let context = SymbolicContext::new(&bn).unwrap();
        context.num_parameter_variables()
    }

    /// Set annotation string.
    pub fn set_annotation(&mut self, annotation: &str) {
        self.annotation = annotation.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::observations::Dataset;
    use crate::sketchbook::Sketch;

    #[test]
    /// Test that after processing datasets, they all have matching
    /// variables with model.
    fn processing_dataset() {
        // Build a simple sketch with three variables A, B, C
        let mut sketch = Sketch::from_aeon("A -> A\nB -> B\nC -> C").unwrap();
        assert!(sketch.assert_consistency().is_ok());

        // Valid dataset with the same variables
        let dataset1 = Dataset::new_empty("d1", vec!["A", "B", "C"]).unwrap();
        sketch
            .observations
            .add_dataset_by_str("d1", dataset1)
            .unwrap();

        // Dataset referencing additional non-existing variables D and E
        let dataset2 = Dataset::new_empty("d2", vec!["A", "B", "E", "C", "D"]).unwrap();
        sketch
            .observations
            .add_dataset_by_str("d2", dataset2)
            .unwrap();

        // Dataset missing variables A and C
        let dataset3 = Dataset::new_empty("d3", vec!["B"]).unwrap();
        sketch
            .observations
            .add_dataset_by_str("d3", dataset3)
            .unwrap();

        // Dataset that is both missing variable C and has extra variable E
        let dataset4 = Dataset::new_empty("d4", vec!["A", "B", "E"]).unwrap();
        sketch
            .observations
            .add_dataset_by_str("d4", dataset4)
            .unwrap();

        let sketch_processed = sketch.with_processed_datasets();
        assert_eq!(sketch_processed.observations.num_datasets(), 4);

        let new_d1 = sketch_processed.observations.get_dataset_by_str("d1");
        let new_d2 = sketch_processed.observations.get_dataset_by_str("d2");
        let new_d3 = sketch_processed.observations.get_dataset_by_str("d3");
        let new_d4 = sketch_processed.observations.get_dataset_by_str("d4");

        // Datasets d1, d2, and d4 should all look like the original d1
        // Dataset d3 should have different variable order
        let expected_d1 = Dataset::new_empty("d1", vec!["A", "B", "C"]).unwrap();
        let expected_d2 = Dataset::new_empty("d2", vec!["A", "B", "C"]).unwrap();
        let expected_d3 = Dataset::new_empty("d3", vec!["B", "A", "C"]).unwrap();
        let expected_d4 = Dataset::new_empty("d4", vec!["A", "B", "C"]).unwrap();

        assert_eq!(new_d1.unwrap(), &expected_d1);
        assert_eq!(new_d2.unwrap(), &expected_d2);
        assert_eq!(new_d3.unwrap(), &expected_d3);
        assert_eq!(new_d4.unwrap(), &expected_d4);
    }
}
