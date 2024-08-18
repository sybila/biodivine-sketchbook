use crate::sketchbook::model::ModelState;
use biodivine_lib_param_bn::{BooleanNetwork, RegulatoryGraph};

/// Methods for converting between `ModelState` and `BooleanNetwork` (from the `lib-param-bn`).
impl ModelState {
    /// Internal function to convert the `ModelState` into a variant of `BooleanNetwork` with
    /// specified information to be included.
    ///
    /// By default, all variables and regulations are included. You can choose the following:
    /// - `regulation_types`: include types of regulations
    /// - `parameters`: include all parameters for uninterpreted functions
    /// - `update_fns`: include all update functions
    /// - `extra_vars`: additional extra variables (with no update fns, no regulations)
    ///
    /// It is up to you to make the selection reasonable (e.g., when including update functions
    /// that contain parameters, you must also include parameters, and so on...).
    fn to_bn_with_options(
        &self,
        regulation_types: bool,
        parameters: bool,
        update_fns: bool,
        extra_vars: Option<Vec<String>>,
    ) -> Result<BooleanNetwork, String> {
        let reg_graph = if regulation_types {
            self.to_reg_graph(extra_vars)
        } else {
            self.to_reg_graph_with_unspecified_regs(extra_vars)
        };
        let mut bn = BooleanNetwork::new(reg_graph);

        if parameters {
            for (fn_id, uninterpreted_fn) in self.uninterpreted_fns.iter() {
                // uninterpreted fns always have unique valid IDs, so we can unwrap
                bn.add_parameter(fn_id.as_str(), uninterpreted_fn.get_arity() as u32)
                    .unwrap();
            }
        }

        if update_fns {
            for (var_id, update_fn) in self.update_fns.iter() {
                if !update_fn.is_unspecified() {
                    bn.add_string_update_function(var_id.as_str(), update_fn.get_fn_expression())?;
                }
            }
        }
        Ok(bn)
    }

    /// Convert the `ModelState` into the corresponding "default" `BooleanNetwork` object.
    /// The resulting BN covers the variables and regulations, but it has empty update functions,
    /// and does not cover any parameters.
    pub fn to_empty_bn(&self) -> BooleanNetwork {
        // this is a safe combination that cannot result in errors
        self.to_bn_with_options(true, false, false, None).unwrap()
    }

    /// Convert the `ModelState` into the corresponding "default" `BooleanNetwork` object with added
    /// parameters.
    /// The resulting BN covers the variables, parameters, and regulations, but it has empty update functions.
    pub fn to_empty_bn_with_params(&self) -> BooleanNetwork {
        // this is a safe combination that cannot result in errors
        self.to_bn_with_options(true, true, false, None).unwrap()
    }

    /// Generate a `BooleanNetwork` with a only given number of "placeholder" (fake) variables.
    /// These variables will be named `var0`, `var1`, ...
    ///
    /// The resulting BN will normally contain all uninterpreted functions (parameters) of this model.
    /// There will be no regulations, and update functions will be empty.
    ///
    /// This is useful for parsing `FnUpdate` objects describing syntactic trees of uninterpreted functions.
    pub fn to_fake_bn_with_params(&self, num_variables: usize) -> BooleanNetwork {
        // construct a bn with fake variables
        let fake_vars = (0..num_variables).map(|i| format!("var{i}")).collect();
        let reg_graph = RegulatoryGraph::new(fake_vars);
        let mut bn = BooleanNetwork::new(reg_graph);

        // add all the parameters
        for (fn_id, uninterpreted_fn) in self.uninterpreted_fns.iter() {
            // uninterpreted fns always have unique valid IDs, so we can unwrap
            bn.add_parameter(fn_id.as_str(), uninterpreted_fn.get_arity() as u32)
                .unwrap();
        }
        bn
    }

    /// Convert the `ModelState` into the corresponding `BooleanNetwork` object (that will contain all of the
    /// variables, regulations, update functions, and uninterpreted functions.
    ///
    /// Note that currently the `BooleanNetwork` class does not support all features of the `ModelState` (such as
    /// various regulation types or details of uninterpreted functions) -- these will be lost during the conversion.
    pub fn to_bn(&self) -> BooleanNetwork {
        // this is a safe combination that cannot result in errors
        self.to_bn_with_options(true, true, true, None).unwrap()
    }

    /// Convert the `ModelState` into the corresponding `BooleanNetwork` object (that will contain all of the
    /// variables, (plain) regulations, update functions, and uninterpreted functions). However,
    /// the types of regulations (both monotonicity and essentiality) are ignored, and instead used as
    /// unspecified.
    ///
    /// This might be useful if we want to process regulation types later via static properties.
    ///
    /// You can add optional extra variables (`extra_vars`). These will have no update fns or
    /// regulations.
    pub fn to_bn_with_plain_regulations(&self, extra_vars: Option<Vec<String>>) -> BooleanNetwork {
        // this is a safe combination that cannot result in errors
        self.to_bn_with_options(false, true, true, extra_vars)
            .unwrap()
    }
}

impl ModelState {
    /// Convert the `ModelState` into the corresponding `BooleanNetwork` object (that will contain
    /// all of the variables, regulations, update functions, and uninterpreted functions).
    ///
    /// See [ModelState::from_reg_graph] for details on how regulations and variables are handled.
    ///
    /// A name of parameters used in BooleanNetwork (which should be unique) is used as both
    /// its ID and name in the resulting ModelState.
    pub fn from_bn(bn: &BooleanNetwork) -> Result<Self, String> {
        // this collects variables and regulations
        let mut model = ModelState::from_reg_graph(bn.as_graph())?;

        // add parameters
        for param_id in bn.parameters() {
            let param = bn.get_parameter(param_id);
            let name = param.get_name();
            model.add_uninterpreted_fn_by_str(name, name, param.get_arity() as usize)?;
        }

        // and also add update functions
        for var in bn.variables() {
            let var_name = bn.get_variable_name(var);
            let update_fn_opt = bn.get_update_function(var);
            if let Some(update_fn) = update_fn_opt {
                let var_id = model.get_var_id(var_name)?;
                model.set_update_fn(&var_id, &update_fn.to_string(bn))?;
            }
        }
        Ok(model)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::model::ModelState;

    /// Prepare a test model containing all the different components.
    pub(super) fn prepare_test_model_full() -> ModelState {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        let var_a = model.get_var_id("a").unwrap();
        model
            .add_multiple_regulations(vec!["a -> b", "b -> a", "a -| a"])
            .unwrap();
        model.add_uninterpreted_fn_by_str("f", "f", 2).unwrap();
        model.set_update_fn(&var_a, "b & !a").unwrap();
        model
    }

    #[test]
    fn test_to_empty_bn() {
        let model = prepare_test_model_full();
        let bn = model.to_empty_bn();
        let var_a = bn.as_graph().find_variable("a").unwrap();
        let var_b = bn.as_graph().find_variable("b").unwrap();
        assert_eq!(bn.num_vars(), 2);
        assert_eq!(bn.regulators(var_a), vec![var_a, var_b]);
        assert_eq!(bn.num_parameters(), 0);
    }

    #[test]
    fn test_to_empty_bn_with_params() {
        let model = prepare_test_model_full();
        let bn = model.to_empty_bn_with_params();
        assert_eq!(bn.num_parameters(), 1);
        assert!(bn.find_parameter("f").is_some());
    }

    #[test]
    fn test_to_bn_and_back() {
        let model = prepare_test_model_full();
        let bn = model.to_bn();
        let var_a = bn.as_graph().find_variable("a").unwrap();
        let var_b = bn.as_graph().find_variable("b").unwrap();
        let model_var_a = model.get_var_id("a").unwrap();
        let update_var_a = model.get_update_fn(&model_var_a).unwrap().to_fn_update(&bn);
        assert_eq!(bn.get_update_function(var_a), &update_var_a);
        assert_eq!(bn.get_update_function(var_b), &None);

        // the conversion back works only if IDs and names (for variables, functions) match (which
        // is out case)
        let model_converted = ModelState::from_bn(&bn).unwrap();
        assert_eq!(model, model_converted);
    }

    #[test]
    fn test_to_fake_bn() {
        let model = prepare_test_model_full();
        let bn = model.to_fake_bn_with_params(2);
        let var_0 = bn.as_graph().find_variable("var0").unwrap();
        let var_1 = bn.as_graph().find_variable("var1").unwrap();
        assert_eq!(bn.num_vars(), 2);
        assert_eq!(bn.as_graph().regulations().len(), 0);
        assert_eq!(bn.num_parameters(), 1);
        assert_eq!(bn.get_update_function(var_0), &None);
        assert_eq!(bn.get_update_function(var_1), &None);
    }
}
