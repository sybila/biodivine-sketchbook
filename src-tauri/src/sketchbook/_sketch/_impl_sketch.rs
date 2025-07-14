use biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext;

use crate::sketchbook::data_structs::SketchData;
use crate::sketchbook::model::ModelState;
use crate::sketchbook::observations::{Dataset, ObservationManager};
use crate::sketchbook::properties::{DynProperty, PropertyManager, StatProperty};
use crate::sketchbook::Sketch;

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
