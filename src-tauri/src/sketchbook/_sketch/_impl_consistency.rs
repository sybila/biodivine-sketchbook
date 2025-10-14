use crate::sketchbook::ids::{DatasetId, ObservationId, UninterpretedFnId, VarId};
use crate::sketchbook::properties::dynamic_props::{
    DynPropertyType, WildCardProposition, WildCardType,
};
use crate::sketchbook::properties::static_props::StatPropertyType;
use crate::sketchbook::properties::{DynProperty, FirstOrderFormula, HctlFormula, StatProperty};
use crate::sketchbook::Sketch;
use std::collections::HashSet;

/// Utilities to perform consistency checks.
impl Sketch {
    /// Assert that the sketch is consistent, return error otherwise.
    /// See [Self::run_consistency_check] for details on which criteria are checked.
    pub fn assert_consistency(&self) -> Result<(), String> {
        if self.run_consistency_check().0 {
            Ok(())
        } else {
            Err("Sketch is not consistent.".to_string())
        }
    }

    /// General check that all components of the sketch are consistent together.
    /// Returns a flag whether the sketch is consistent, a main message summarizing potential
    /// major issues (or stating there are none), and a message with warnings. Warnings
    /// are smaller issues that do not make sketch inconsistent, but may help users.
    ///
    /// Note that most of the general consistency (syntax of formulas, check validity and
    /// uniqueness of IDs, ..) is enforced automatically when editing the sketch. However,
    /// some more complex details are left to be checked (either explicitely or automatically
    /// before the inference is started).
    ///
    /// This should include:
    /// - check that model is not empty, and there are no problems in function expressions
    /// - check that dataset variables are valid network variables and vice versa
    /// - check that various template properties reference valid variables and data
    /// - check that HCTL formulas only use valid variables as atomic propositions
    /// - check that FOL formulas only use valid function symbols
    pub fn run_consistency_check(&self) -> (bool, String, String) {
        let mut all_consitent = true;
        let mut main_message = String::new();
        // A message with less important issues, will be sent as warnings.
        // Warnings do not affect overall consistency.
        let mut warning_message = String::new();

        // we divide the code by different components to avoid replication
        let componets_results = vec![
            self.check_model(),
            self.check_datasets(),
            self.check_static(),
            self.check_dynamic(),
        ];

        for (consistent, sub_err_message, sub_warn_message) in componets_results {
            if !consistent {
                main_message += sub_err_message.as_str();
                main_message += "\n";
                all_consitent = false;
            }
            if !sub_warn_message.is_empty() {
                warning_message += sub_warn_message.as_str();
                warning_message += "\n";
            }
        }
        (all_consitent, main_message, warning_message)
    }

    /// Part of the consistency check responsible for the 'model' component.
    /// Returns bool (whether a model is consistent), a formated message with error issues,
    /// and a separate message with warnings.
    ///
    /// We currently ensure that the network is not empty, there are no redundant (unused)
    /// function symbols, and that expressions of uninterpreted functions are not defined
    /// recursively.
    fn check_model(&self) -> (bool, String, String) {
        let mut consitent = true;
        let mut message = String::new();
        message += "MODEL:\n";

        // check model is not empty
        if self.model.num_vars() == 0 {
            consitent = false;
            message += "> ISSUE: There must be at least one variable.\n";
        }

        // Check there are no cycles in expressions of uninterpreted functions (e.g., having
        // that `f(x) = g(x)` and `g(x) = f(x) | !x`). Circular definitions are not allowed.
        if let Err(error_msg) = self.model.assert_no_cycles_in_fn_expressions() {
            consitent = false;
            // The particular function is mentioned in the error message itself
            let issue = format!("> ISSUE: Function expressions cannot be recursive: {error_msg}\n");
            message += &issue;
        }

        // Check there are no uninterpreted functions that would be completely unused in any update
        // expression, even after propagating function expressions. If the method returns an error, it
        // is because there are recursive definitions (which is already reported in the previous step).
        if let Ok(redundant_fn_symbols) = self.model.find_redundant_uninterpreted_fns() {
            for fn_symbol in redundant_fn_symbols {
                consitent = false;
                let issue = format!(
                    "> ISSUE: Function `{fn_symbol}` is redundant (not used in any update expression). Consider using it or remove it.\n"
                );
                message += &issue;
            }
        }

        // TODO: Maybe allow the redundant functions? We already check if these symbols are not
        //       used in static properties, and we prune the rest later, so it should be fine.

        // TODO: We can consider adding a check whether update fn expressions match regulation
        //       properties (essentially a partial check for some static properties)

        // At the moment, there are no warnings here
        (consitent, message, String::new())
    }

    /// Part of the consistency check responsible for the 'observations' (datasets) component.
    /// Returns bool (whether datasets are consistent), a formated message with error issues,
    /// and a separate message with warnings.
    ///
    /// Essentially, we currently check that variables in datasets and in the network exactly match.
    fn check_datasets(&self) -> (bool, String, String) {
        let mut message = String::new();
        message += "DATASETS:\n";

        // Warning message with minor issues to warn user
        // For now, this is just regarding unused datasets
        let mut warnings = String::new();

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
            // check that all network variables are part of the dataset
            // TODO: in future, maybe automatically add missing variables before inference instead?
            for (var_id, _) in self.model.variables() {
                if !dataset.is_valid_variable(var_id) {
                    let issue_var =
                        format!("Variable {var_id} is not present in the dataset {dataset_id}.");
                    let issue = format!("> ISSUE with dataset `{dataset_id}`: {issue_var}\n");
                    message += &issue;
                    dataset_err_found = true;
                }
            }

            // Check if the dataset is used within some dynamic prop, and if not, create
            // warning
            if !self.is_dataset_used(dataset_id) {
                let warning =
                    format!("Dataset {dataset_id} is not used in any dynamic properties.\n");
                warnings += &warning;
            }
        }

        (!dataset_err_found, message, warnings)
    }

    /// Part of the consistency check responsible for the 'static properties' component.
    /// Returns bool (whether a static properties are consistent), a formated message with error issues,
    /// and a separate message with warnings.
    fn check_static(&self) -> (bool, String, String) {
        let mut message = String::new();
        message += "STATIC PROPERTIES:\n";

        let mut stat_err_found = false;
        for (prop_id, prop) in self.properties.stat_props() {
            if let Err(e) = self.assert_static_prop_valid(prop) {
                message = append_property_issue(&e, prop_id.as_str(), message);
                stat_err_found = true;
            }
        }
        // At the moment, there are no warnings here
        (!stat_err_found, message, String::new())
    }

    /// Part of the consistency check responsible for the 'dynamic properties' component.
    /// Returns bool (whether a dynamic properties are consistent), a formated message with error issues,
    /// and a separate message with warnings.
    fn check_dynamic(&self) -> (bool, String, String) {
        let mut message = String::new();
        message += "DYNAMIC PROPERTIES:\n";

        let mut dyn_err_found = false;
        for (prop_id, prop) in self.properties.dyn_props() {
            if let Err(e) = self.assert_dynamic_prop_valid(prop) {
                message = append_property_issue(&e, prop_id.as_str(), message);
                dyn_err_found = true;
            }
        }
        // At the moment, there are no warnings here
        (!dyn_err_found, message, String::new())
    }

    /// Check if all fields of the static property are filled and have valid values.
    /// If not, return appropriate message.
    ///
    /// Apart from the syntactic check, static properties should not reference any
    /// "redundant" uninterpreted functions (unused functions that won't appear in
    /// any update expression, even after propagating the function expressions).
    fn assert_static_prop_valid(&self, prop: &StatProperty) -> Result<(), String> {
        // First just check if all required fields are filled out
        prop.assert_fully_filled()?;

        // Collect all unused function IDs that should not be utilized in any property
        // If the function returns error, use empty set (since the issue is handled elsewhere)
        let unused_functions: HashSet<String> = self
            .model
            .find_redundant_uninterpreted_fns()
            .unwrap_or_default()
            .iter()
            .map(|f| f.to_string())
            .collect();

        // Now, let's validate the fields (we know the required ones are filled in)
        match prop.get_prop_data() {
            StatPropertyType::GenericStatProp(generic_prop) => {
                FirstOrderFormula::check_syntax_with_model(&generic_prop.raw_formula, &self.model)?;
                let functions_referenced = generic_prop
                    .processed_formula
                    .tree()
                    .collect_unique_fn_symbols()
                    .unwrap();
                for (fn_id, _) in functions_referenced.iter() {
                    self.assert_fn_symbol_not_redundant(fn_id, &unused_functions)?;
                }
            }
            StatPropertyType::FnInputEssential(p)
            | StatPropertyType::FnInputEssentialContext(p) => {
                let fn_id = p.target.as_ref().unwrap();
                self.assert_fn_valid_in_model(fn_id)?;
                self.assert_fn_index_valid(p.input_index.unwrap(), fn_id)?;
                self.assert_context_valid_or_none(p.context.as_ref())?;
                self.assert_fn_symbol_not_redundant(fn_id.as_str(), &unused_functions)?;
            }
            StatPropertyType::FnInputMonotonic(p)
            | StatPropertyType::FnInputMonotonicContext(p) => {
                let fn_id = p.target.as_ref().unwrap();
                self.assert_fn_valid_in_model(fn_id)?;
                self.assert_fn_index_valid(p.input_index.unwrap(), fn_id)?;
                self.assert_context_valid_or_none(p.context.as_ref())?;
                self.assert_fn_symbol_not_redundant(fn_id.as_str(), &unused_functions)?;
            }
            StatPropertyType::RegulationEssential(p)
            | StatPropertyType::RegulationEssentialContext(p) => {
                self.assert_var_valid_in_model(p.target.as_ref().unwrap())?;
                self.assert_var_valid_in_model(p.input.as_ref().unwrap())?;
                self.assert_context_valid_or_none(p.context.as_ref())?;
            }
            StatPropertyType::RegulationMonotonic(p)
            | StatPropertyType::RegulationMonotonicContext(p) => {
                self.assert_var_valid_in_model(p.target.as_ref().unwrap())?;
                self.assert_var_valid_in_model(p.input.as_ref().unwrap())?;
                self.assert_context_valid_or_none(p.context.as_ref())?;
            }
        }
        Ok(())
    }

    /// Check if all fields of a dynamic property are filled and have valid values.
    /// If not, return appropriate message.
    fn assert_dynamic_prop_valid(&self, prop: &DynProperty) -> Result<(), String> {
        // first just check if all required fields are filled out (that is usually the dataset ID)
        prop.assert_dataset_filled()?;

        // now, let's validate the fields (we know the required ones are filled in)
        match prop.get_prop_data() {
            DynPropertyType::GenericDynProp(generic_prop) => {
                HctlFormula::check_syntax_with_model(
                    generic_prop.processed_formula.as_str(),
                    &self.model,
                )?;

                // we will need to check wild cards as well
                for wild_card_proposition in generic_prop.wild_cards.iter() {
                    self.assert_wild_card_prop_valid(wild_card_proposition)?;
                }
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

    /// Check if all fields of a wild-card proposition are filled and have valid values.
    /// If not, return appropriate message.
    fn assert_wild_card_prop_valid(&self, prop: &WildCardProposition) -> Result<(), String> {
        match prop.get_prop_data() {
            WildCardType::Observation(data_id, obs_id) => {
                self.assert_dataset_valid(data_id)?;
                self.assert_obs_valid_or_none(data_id, Some(obs_id))?;
            }
            WildCardType::Trajectory(data_id) => {
                self.assert_dataset_valid(data_id)?;
            }
            WildCardType::Attractors(data_id, obs_id) => {
                self.assert_dataset_valid(data_id)?;
                self.assert_obs_valid_or_none(data_id, obs_id.as_ref())?;
            }
            WildCardType::FixedPoints(data_id, obs_id) => {
                self.assert_dataset_valid(data_id)?;
                self.assert_obs_valid_or_none(data_id, obs_id.as_ref())?;
            }
            WildCardType::TrapSpaces(data_id, obs_id, _, _) => {
                self.assert_dataset_valid(data_id)?;
                self.assert_obs_valid_or_none(data_id, obs_id.as_ref())?;
            }
            WildCardType::AttractorCount(..) => {} // no fields that can be invalid
        }
        Ok(())
    }

    /// Check that the function symbol (that is referenced in the property) is not
    /// one of the redundant symbols. If not, return error with a proper message.
    ///
    /// Note: redundant symbols are those that won't appear in any update expression,
    /// even after propagating the function expressions.
    fn assert_fn_symbol_not_redundant(
        &self,
        fn_id: &str,
        redundant_functions: &HashSet<String>,
    ) -> Result<(), String> {
        if redundant_functions.contains(fn_id) {
            Err(format!(
                "Property references unused function `{fn_id}` (which is not used in any update expression, and thus not part of the model)."
            ))
        } else {
            Ok(())
        }
    }

    /// Check that variable is valid in a model. If not, return error with a proper message.
    fn assert_var_valid_in_model(&self, var_id: &VarId) -> Result<(), String> {
        if self.model.is_valid_var_id(var_id) {
            Ok(())
        } else {
            let msg = format!("Variable `{var_id}` is not a valid variable in the model.");
            Err(msg)
        }
    }

    /// Check that function is valid in a model. If not, return error with a proper message.
    fn assert_fn_valid_in_model(&self, fn_id: &UninterpretedFnId) -> Result<(), String> {
        if self.model.is_valid_uninterpreted_fn_id(fn_id) {
            Ok(())
        } else {
            let msg = format!("Function `{fn_id}` is not a valid function in the model.");
            Err(msg)
        }
    }

    /// Check that input index of uninterpreted function is in range (smaller than the arity).
    /// If not, return error with a proper message.
    fn assert_fn_index_valid(&self, index: usize, fn_id: &UninterpretedFnId) -> Result<(), String> {
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

    /// Check if the given dataset is used within any of the dyn properties.
    /// We expect dataset ID is already checked as valid.
    fn is_dataset_used(&self, dataset_id: &DatasetId) -> bool {
        for (_, dyn_prop) in self.properties.dyn_props() {
            // If property has dataset subfield and it is filled, we check it for match
            if let Ok(Some(prop_dataset)) = dyn_prop.get_dataset() {
                if &prop_dataset == dataset_id {
                    return true;
                }
            }
        }
        false
    }
}

/// **(internal)** Simple internal utility to append issue message regarding a particular property.
fn append_property_issue(description: &str, prop_id: &str, mut log: String) -> String {
    let issue = format!("> ISSUE with property `{prop_id}`: {description}\n");
    log += &issue;
    log
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::observations::Dataset;
    use crate::sketchbook::properties::{DynProperty, StatProperty};
    use crate::sketchbook::Sketch;
    use std::fs::File;
    use std::io::Read;

    #[test]
    /// Test that consistency check is successful on our test sketch.
    fn consistency_valid_sketch() {
        let mut sketch_file = File::open("../data/test_data/test_sketch_1.json").unwrap();
        let mut file_content = String::new();
        sketch_file.read_to_string(&mut file_content).unwrap();
        let sketch = Sketch::from_custom_json(&file_content).unwrap();
        assert!(sketch.assert_consistency().is_ok());
    }

    #[test]
    /// Test that consistency check reports issues when model is empty (no variables).
    fn consistency_empty_model() {
        let sketch = Sketch::default();
        assert!(sketch.assert_consistency().is_err());
    }

    #[test]
    /// Test that consistency check reports issues if there are recursive definitions in
    /// expressions of uninterpreted functions.
    fn consistency_recursion_in_expressions() {
        // a simple sketch that defines two function symbols
        let mut sketch = Sketch::from_aeon("A -> A\n$A:f(A) | g(A)").unwrap();
        // the expressions of the functions have cyclic references
        sketch
            .model
            .set_uninterpreted_fn_expression_by_str("f", "g(var0) | var0")
            .unwrap();
        sketch
            .model
            .set_uninterpreted_fn_expression_by_str("g", "!var0 & f(var0)")
            .unwrap();
        assert!(sketch.assert_consistency().is_err());
    }

    #[test]
    /// Test that consistency check reports issues if there are redundant function symbols.
    fn consistency_redundant_function() {
        let mut sketch = Sketch::from_aeon("A -> A\n$A:f(A)").unwrap();
        // another function symbol not used anywhere
        sketch
            .model
            .add_empty_uninterpreted_fn_by_str("g", "g", 2)
            .unwrap();
        assert!(sketch.assert_consistency().is_err());
    }

    #[test]
    /// Test that consistency check reports issues if a HCTL/FOL property references variable not
    /// present in the model.
    fn consistency_properties() {
        // build a simple sketch with one variable A and function symbol f
        let sketch = Sketch::from_aeon("A -> A\n$A:f(A)").unwrap();
        assert!(sketch.assert_consistency().is_ok());

        // hctl referencing non-existing variable B
        let hctl_formula = "B";
        let dyn_prop = DynProperty::try_mk_generic("", hctl_formula).unwrap();
        let mut sketch_copy = sketch.clone();
        sketch_copy
            .properties
            .add_dynamic_by_str("p", dyn_prop)
            .unwrap();
        assert!(sketch_copy.assert_consistency().is_err());

        // fol referencing non-existing function g
        let fol_formula = "g(1)";
        let stat_prop = StatProperty::try_mk_generic("", fol_formula).unwrap();
        let mut sketch_copy = sketch.clone();
        sketch_copy
            .properties
            .add_static_by_str("p", stat_prop)
            .unwrap();
        assert!(sketch_copy.assert_consistency().is_err());
    }

    #[test]
    /// Test that property consistency check reports issues if a redundant function symbol is
    /// used.
    fn consistency_property_with_redundant_function() {
        let mut sketch = Sketch::from_aeon("A -> A\n$A:f(A)").unwrap();
        // add another function symbol not used anywhere
        sketch
            .model
            .add_empty_uninterpreted_fn_by_str("g", "g", 1)
            .unwrap();
        let static_prop = StatProperty::try_mk_generic("prop", "\\exists x: g(x)").unwrap();
        sketch
            .properties
            .add_static_by_str("prop", static_prop.clone())
            .unwrap();

        // We cant use the full consistency check directly - if would report error as well,
        // but simply because there is the unused function in the sketch (different reason).
        assert!(sketch.assert_static_prop_valid(&static_prop).is_err());
    }

    #[test]
    /// Test that consistency check fails if a dataset contains variable not
    /// present in the model (and the other way around).
    fn consistency_dataset() {
        // build a simple sketch with one variables A, B and a function symbol f
        let sketch = Sketch::from_aeon("A -> A\nB -> B\n$A:f(A)\n$B:f(B)").unwrap();
        assert!(sketch.assert_consistency().is_ok());

        // dataset referencing additional non-existing variable C
        let dataset = Dataset::new_empty("d1", vec!["A", "B", "C"]).unwrap();
        let mut sketch_copy = sketch.clone();
        sketch_copy
            .observations
            .add_dataset_by_str("d1", dataset)
            .unwrap();
        assert!(sketch_copy.assert_consistency().is_err());

        // dataset missing variable B
        let dataset = Dataset::new_empty("d2", vec!["A"]).unwrap();
        let mut sketch_copy = sketch.clone();
        sketch_copy
            .observations
            .add_dataset_by_str("d2", dataset)
            .unwrap();
        assert!(sketch_copy.assert_consistency().is_err());

        // dataset that is consistent with the model
        let dataset = Dataset::new_empty("d3", vec!["A", "B"]).unwrap();
        let mut sketch_copy = sketch.clone();
        sketch_copy
            .observations
            .add_dataset_by_str("d3", dataset)
            .unwrap();
        assert!(sketch_copy.assert_consistency().is_ok());
    }
}
