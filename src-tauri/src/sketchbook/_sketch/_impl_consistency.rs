use crate::sketchbook::properties::dynamic_props::DynPropertyType;
use crate::sketchbook::properties::static_props::StatPropertyType;
use crate::sketchbook::properties::{FirstOrderFormula, HctlFormula};
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
    /// - check that various template properties only use valid variables and data
    /// - check that HCTL properties only use valid variables as atomic propositions
    /// - check that FOL properties only use valid function symbols
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
                all_consitent = false;
            }
        }

        (all_consitent, message)
    }

    fn check_model(&self) -> (bool, String) {
        let mut consitent = true;
        let mut message = String::new();
        message += "MODEL:\n";

        if self.model.num_vars() == 0 {
            consitent = false;
            message += "> ISSUE: There must be at least one variable.\n";
        }

        // TODO: in future, we can add check whether update fns match regulation monotonicity
        (consitent, message)
    }

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

    fn check_static(&self) -> (bool, String) {
        let mut message = String::new();
        message += "STATIC PROPERTIES:\n";

        let mut stat_err_found = false;
        for (prop_id, prop) in self.properties.stat_props() {
            if let StatPropertyType::GenericStatProp(generic_prop) = prop.get_prop_data() {
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
        (!stat_err_found, message)
    }

    fn check_dynamic(&self) -> (bool, String) {
        let mut message = String::new();
        message += "DYNAMIC PROPERTIES:\n";

        let mut dyn_err_found = false;
        for (prop_id, prop) in self.properties.dyn_props() {
            if let DynPropertyType::GenericDynProp(generic_prop) = prop.get_prop_data() {
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
        (!dyn_err_found, message)
    }
}
