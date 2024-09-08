use crate::sketchbook::data_structs::SketchData;
use crate::sketchbook::model::ModelState;
use crate::sketchbook::observations::{Dataset, ObservationManager};
use crate::sketchbook::properties::dynamic_props::DynPropertyType;
use crate::sketchbook::properties::static_props::StatPropertyType;
use crate::sketchbook::properties::{
    DynProperty, FirstOrderFormula, HctlFormula, PropertyManager, StatProperty,
};
use crate::sketchbook::Sketch;

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

    /// Assert that the sketch is consistent, return error otherwise.
    /// See [run_consistency_check] for details on which criteria are checked.
    pub fn assert_consistency(&self) -> Result<(), String> {
        if self.run_consistency_check().0 {
            Ok(())
        } else {
            Err("Sketch is not consistent.".to_string())
        }
    }

    /// General check that all components of the sketch are consistent together.
    /// This should include:
    /// - check that dataset variables are valid network variables
    /// - check that template properties only use valid variables and data
    /// - check that HCTL properties only use valid variables as atomic propositions
    /// - check that FOL properties only use valid function symbols
    pub fn run_consistency_check(&self) -> (bool, String) {
        let mut all_consitent = true;
        let mut message = String::new();
        message += "MODEL:\n";
        if self.model.num_vars() == 0 {
            all_consitent = false;
            message += "> ISSUE: There must be at least one variable.\n";
        }
        // todo
        message += "(this part is not fully implemented yet)\n\n";

        message += "DATASET:\n";
        // todo
        message += "(this part is not fully implemented yet)\n\n";

        message += "STATIC PROPERTIES:\n";
        let mut stat_err_found = false;
        let mut at_least_one_stat_generic = false;
        for (prop_id, prop) in self.properties.stat_props() {
            if let StatPropertyType::GenericStatProp(generic_prop) = prop.get_prop_data() {
                at_least_one_stat_generic = true;
                if let Err(e) = FirstOrderFormula::check_syntax_with_model(
                    &generic_prop.raw_formula,
                    &self.model,
                ) {
                    let issue = format!("> ISSUE with property `{}`: {e}\n", prop_id.as_str());
                    message += &issue;
                    stat_err_found = true;
                }
            }
            // TODO: rest is not implemented yet
        }
        if at_least_one_stat_generic && !stat_err_found {
            message += "> No issues with Generic static properties found.\n";
        }
        message += "(this part is not fully implemented yet)\n\n";
        all_consitent = all_consitent && !stat_err_found;

        message += "DYNAMIC PROPERTIES:\n";
        let mut dyn_err_found = false;
        let mut at_least_one_dyn_generic = false;
        for (prop_id, prop) in self.properties.dyn_props() {
            if let DynPropertyType::GenericDynProp(generic_prop) = prop.get_prop_data() {
                at_least_one_dyn_generic = true;
                if let Err(e) =
                    HctlFormula::check_syntax_with_model(&generic_prop.raw_formula, &self.model)
                {
                    let issue = format!("> ISSUE with property `{}`: {e}\n", prop_id.as_str());
                    message += &issue;
                    dyn_err_found = true;
                }
            }
            // TODO: rest is not implemented yet
        }
        if at_least_one_dyn_generic && !dyn_err_found {
            message += "> No issues with Generic dynamic properties found.\n";
        }
        message += "(this part is not fully implemented yet)\n\n";
        all_consitent = all_consitent && !dyn_err_found;

        (all_consitent, message)
    }
}
