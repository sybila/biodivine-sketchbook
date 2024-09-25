use crate::sketchbook::ids::{DatasetId, ObservationId, UninterpretedFnId, VarId};
use crate::sketchbook::properties::dynamic_props::DynPropertyType;
use crate::sketchbook::properties::static_props::StatPropertyType;
use crate::sketchbook::properties::{DynProperty, FirstOrderFormula, HctlFormula, StatProperty};
use crate::sketchbook::Sketch;

/// Utilities to perform consistency checks.
impl Sketch {
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
    ///
    /// Note that most of the general consistency (syntax of formulas, check validity and
    /// uniqueness of IDs, ..) is enforced automatically when editing the sketch. However,
    /// some more complex details are left to be checked (explicitely or before analysis).
    ///
    /// This should include:
    /// - check that model is not empty
    /// - check that dataset variables are valid network variables
    /// - check that various template properties reference valid variables and data
    /// - check that HCTL formulas only use valid variables as atomic propositions
    /// - check that FOL formulas only use valid function symbols
    pub fn run_consistency_check(&self) -> (bool, String) {
        let mut all_consitent = true;
        let mut message = String::new();

        // we divide the code by different components to avoid replication
        let componets_results = vec![
            self.check_model(),
            self.check_datasets(),
            self.check_static(),
            self.check_dynamic(),
        ];

        for (consistent, sub_message) in componets_results {
            if !consistent {
                message += sub_message.as_str();
                message += "\n";
                all_consitent = false;
            }
        }

        (all_consitent, message)
    }

    /// Part of the consistency check responsible for the 'model' component.
    /// Returns bool (whether a model is consistent) and formated message with issues.
    fn check_model(&self) -> (bool, String) {
        let mut consitent = true;
        let mut message = String::new();
        message += "MODEL:\n";

        if self.model.num_vars() == 0 {
            consitent = false;
            message += "> ISSUE: There must be at least one variable.\n";
        }

        // TODO: in future, we can also add a check whether update fns match regulation monotonicity

        (consitent, message)
    }

    /// Part of the consistency check responsible for the 'observations' (datasets) component.
    /// Returns bool (whether a datasets are consistent) and formated message with issues.
    fn check_datasets(&self) -> (bool, String) {
        let mut message = String::new();
        message += "DATASETS:\n";

        let mut dataset_err_found = false;
        for (dataset_id, dataset) in self.observations.datasets() {
            // check that all dataset variables are part of the network
            for var_id in dataset.variables() {
                if !self.model.is_valid_var_id(var_id) {
                    let issue_var =
                        format!("Variable {} is not present in the model.", var_id.as_str());
                    let issue = format!(
                        "> ISSUE with dataset `{}`: {issue_var}\n",
                        dataset_id.as_str()
                    );
                    message += &issue;
                    dataset_err_found = true;
                }
            }
        }
        (!dataset_err_found, message)
    }

    /// Part of the consistency check responsible for the 'static properties' component.
    /// Returns bool (whether a static properties are consistent) and formated message with issues.
    fn check_static(&self) -> (bool, String) {
        let mut message = String::new();
        message += "STATIC PROPERTIES:\n";

        let mut stat_err_found = false;
        for (prop_id, prop) in self.properties.stat_props() {
            if let Err(e) = self.assert_static_prop_valid(prop) {
                message = append_property_issue(&e, prop_id.as_str(), message);
                stat_err_found = true;
            }
        }
        (!stat_err_found, message)
    }

    /// Part of the consistency check responsible for the 'dynamic properties' component.
    /// Returns bool (whether a dynamic properties are consistent) and formated message with issues.
    fn check_dynamic(&self) -> (bool, String) {
        let mut message = String::new();
        message += "DYNAMIC PROPERTIES:\n";

        let mut dyn_err_found = false;
        for (prop_id, prop) in self.properties.dyn_props() {
            if let Err(e) = self.assert_dynamic_prop_valid(prop) {
                message = append_property_issue(&e, prop_id.as_str(), message);
                dyn_err_found = true;
            }
        }
        (!dyn_err_found, message)
    }

    /// Check if all fields of the static property are filled and have valid values.
    /// If not, return appropriate message.
    fn assert_static_prop_valid(&self, prop: &StatProperty) -> Result<(), String> {
        // first just check if all required fields are filled out
        prop.assert_fully_filled()?;

        // now, let's validate the fields (we know the required ones are filled in)
        match prop.get_prop_data() {
            StatPropertyType::GenericStatProp(generic_prop) => {
                FirstOrderFormula::check_syntax_with_model(&generic_prop.raw_formula, &self.model)?;
            }
            StatPropertyType::FnInputEssential(p)
            | StatPropertyType::FnInputEssentialContext(p) => {
                self.assert_fn_valid(p.target.as_ref().unwrap())?;
                self.assert_index_valid(p.input_index.unwrap(), p.target.as_ref().unwrap())?;
                self.assert_context_valid_or_none(p.context.as_ref())?;
            }
            StatPropertyType::FnInputMonotonic(p)
            | StatPropertyType::FnInputMonotonicContext(p) => {
                self.assert_fn_valid(p.target.as_ref().unwrap())?;
                self.assert_index_valid(p.input_index.unwrap(), p.target.as_ref().unwrap())?;
                self.assert_context_valid_or_none(p.context.as_ref())?;
            }
            StatPropertyType::RegulationEssential(p)
            | StatPropertyType::RegulationEssentialContext(p) => {
                self.assert_var_valid(p.target.as_ref().unwrap())?;
                self.assert_var_valid(p.input.as_ref().unwrap())?;
                self.assert_context_valid_or_none(p.context.as_ref())?;
            }
            StatPropertyType::RegulationMonotonic(p)
            | StatPropertyType::RegulationMonotonicContext(p) => {
                self.assert_var_valid(p.target.as_ref().unwrap())?;
                self.assert_var_valid(p.input.as_ref().unwrap())?;
                self.assert_context_valid_or_none(p.context.as_ref())?;
            }
        }
        Ok(())
    }

    /// Check if all fields of the dynamic property are filled and have valid values.
    /// If not, return appropriate message.
    fn assert_dynamic_prop_valid(&self, prop: &DynProperty) -> Result<(), String> {
        // first just check if all required fields are filled out (that is usually the dataset ID)
        prop.assert_dataset_filled()?;

        // now, let's validate the fields (we know the required ones are filled in)
        match prop.get_prop_data() {
            DynPropertyType::GenericDynProp(generic_prop) => {
                HctlFormula::check_syntax_with_model(&generic_prop.raw_formula, &self.model)?;
            }
            DynPropertyType::HasAttractor(p) => {
                self.assert_dataset_valid(p.dataset.as_ref().unwrap())?;
                self.assert_obs_valid_or_none(p.dataset.as_ref().unwrap(), p.observation.as_ref())?;
            }
            DynPropertyType::ExistsFixedPoint(p) => {
                self.assert_dataset_valid(p.dataset.as_ref().unwrap())?;
                self.assert_obs_valid_or_none(p.dataset.as_ref().unwrap(), p.observation.as_ref())?;
            }
            DynPropertyType::ExistsTrajectory(p) => {
                self.assert_dataset_valid(p.dataset.as_ref().unwrap())?;
            }
            DynPropertyType::ExistsTrapSpace(p) => {
                self.assert_dataset_valid(p.dataset.as_ref().unwrap())?;
                self.assert_obs_valid_or_none(p.dataset.as_ref().unwrap(), p.observation.as_ref())?;
            }
            DynPropertyType::AttractorCount(_) => {} // no fields that can be invalid
        }
        Ok(())
    }

    /// Check that variable is valid in a model. If not, return error with a proper message.
    fn assert_var_valid(&self, var_id: &VarId) -> Result<(), String> {
        if self.model.is_valid_var_id(var_id) {
            Ok(())
        } else {
            let msg = format!("Variable `{var_id}` is not a valid variable in the model.");
            Err(msg)
        }
    }

    /// Check that function is valid in a model. If not, return error with a proper message.
    fn assert_fn_valid(&self, fn_id: &UninterpretedFnId) -> Result<(), String> {
        if self.model.is_valid_uninterpreted_fn_id(fn_id) {
            Ok(())
        } else {
            let msg = format!("Function `{fn_id}` is not a valid function in the model.");
            Err(msg)
        }
    }

    /// Check that input index of uninterpreted function is in range (smaller than the arity).
    /// If not, return error with a proper message.
    fn assert_index_valid(&self, index: usize, fn_id: &UninterpretedFnId) -> Result<(), String> {
        let arity = self.model.get_uninterpreted_fn_arity(fn_id)?;
        if arity <= index {
            let msg =
                format!("Function `{fn_id}` has arity {arity}, input index {index} is invalid.");
            return Err(msg);
        }
        Ok(())
    }

    /// Check that context formula is valid. If not, return error with a proper message.
    ///
    /// The context can also be None, which is valid.
    fn assert_context_valid_or_none(&self, context: Option<&String>) -> Result<(), String> {
        if let Some(context_str) = context {
            if let Err(e) = FirstOrderFormula::check_syntax_with_model(context_str, &self.model) {
                let msg = format!("Invalid context formula. {e}");
                return Err(msg);
            }
        }
        Ok(())
    }

    /// Check that dataset is valid. If not, return error with a proper message.
    fn assert_dataset_valid(&self, dataset_id: &DatasetId) -> Result<(), String> {
        if self.observations.is_valid_dataset_id(dataset_id) {
            Ok(())
        } else {
            Err(format!("Dataset `{dataset_id}` is not a valid dataset."))
        }
    }

    /// Check whether observation is valid in a dataset. If not, return error with a proper message.
    /// If observation is None, that is also fine.
    fn assert_obs_valid_or_none(
        &self,
        dataset_id: &DatasetId,
        obs_id: Option<&ObservationId>,
    ) -> Result<(), String> {
        if let Some(obs) = obs_id {
            let dataset = self.observations.get_dataset(dataset_id)?;
            if !dataset.is_valid_obs(obs) {
                let msg = format!("Observation `{obs}` is not valid in dataset `{dataset_id}`.");
                return Err(msg);
            }
        }
        Ok(())
    }
}

/// **(internal)** Simple internal utility to append issue message regarding a particular property.
fn append_property_issue(description: &str, prop_id: &str, mut log: String) -> String {
    let issue = format!("> ISSUE with property `{}`: {description}\n", prop_id);
    log += &issue;
    log
}
