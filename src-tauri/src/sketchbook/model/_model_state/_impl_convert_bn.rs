use crate::sketchbook::model::ModelState;
use biodivine_lib_param_bn::{BooleanNetwork, RegulatoryGraph};

/// Methods for converting between `ModelState` and `BooleanNetwork` (from the `lib-param-bn`).
impl ModelState {
    /// Internal function to convert the `ModelState` into a variant of `BooleanNetwork` with
    /// specified information to be included.
    ///
    /// By default, only "influence graph info" is included: variables and (plain) regulations.
    /// You can further specify to include the following:
    /// - `regulation_types`: include properties of regulations (monotonicity, essentiality)
    /// - `parameters`: include uninterpreted functions as network parameters
    /// - `update_fns`: include update function expressions
    /// - `extra_vars`: add additional extra variables (with no update fns, no regulations)
    ///
    /// It is up to you to make the selection reasonable (e.g., when including update functions
    /// that contain parameters, you must also include parameters, and so on...).
    /// Rather use one of the provided wrappers. For instance, you can get the full PSBN with
    /// all info using [Self::to_bn].
    fn to_bn_internal(
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
                if !update_fn.has_empty_expression() {
                    let mut transformed_update_fn_tree = update_fn.get_fn_tree().clone().unwrap();
                    // TODO: check if this fn contains any function symbols that have their expressions specified
                    let fn_symbols_used = update_fn.collect_fn_symbols();
                    for fn_id in fn_symbols_used {
                        let uninterpreted_fn = self.get_uninterpreted_fn(&fn_id)?;
                        if let Some(fn_expression_tree) = uninterpreted_fn.get_fn_tree() {
                            // TODO: if there is an expression, we must substitute the symbol (in update fn tree) with the expression
                            // all the magical transformations happen inside this method
                            transformed_update_fn_tree = transformed_update_fn_tree
                                .substitute_fn_symbol_with_expression(&fn_id, fn_expression_tree);
                        }
                    }

                    // TODO: add (potentionally transformed) update function
                    let transformed_expression = transformed_update_fn_tree.to_string(self, None);
                    bn.add_string_update_function(var_id.as_str(), &transformed_expression)?;
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
        self.to_bn_internal(true, false, false, None).unwrap()
    }

    /// Convert the `ModelState` into the corresponding "default" `BooleanNetwork` object with added
    /// parameters.
    /// The resulting BN covers the variables, parameters, and regulations, but it has empty update functions.
    pub fn to_empty_bn_with_params(&self) -> BooleanNetwork {
        // this is a safe combination that cannot result in errors
        self.to_bn_internal(true, true, false, None).unwrap()
    }

    /// Generate a `BooleanNetwork` containing only "placeholder" (fake) variables.
    /// These variables will be named `var0`, `var1`, ...
    ///
    /// The resulting BN will normally contain all uninterpreted functions (parameters) of this model.
    /// There will be no regulations, and update functions will be empty.
    ///
    /// We need this for parsing expressions of uninterpreted functions. Uninterpreted function
    /// expression contain these placeholder variables as their formal arguments. We thus need a
    /// "fake" BN to contain these variables as a context when internally creating `FnUpdate` instances
    /// describing the syntactic tree.
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
        self.to_bn_internal(true, true, true, None).unwrap()
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
        self.to_bn_internal(false, true, true, extra_vars).unwrap()
    }
}

impl ModelState {
    /// Convert the `ModelState` into the corresponding `BooleanNetwork` object (that will contain
    /// all of the variables, regulations, update functions, and uninterpreted functions).
    ///
    /// Annotations for both variables and functions are let empty.
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
            model.add_empty_uninterpreted_fn_by_str(name, name, param.get_arity() as usize)?;
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

    /// Prepare a test model used across the tests here.
    /// The model is containing all the different components:
    /// - variables `a`, `b`
    /// - regulations `a -> b`, `b -> a`, `a -| a`
    /// - function symbol `f` of arity 2
    /// - `a` has update `(b & !a) | f(a, b)`, `b` has empty update
    fn prepare_test_model_full() -> ModelState {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        let var_a = model.get_var_id("a").unwrap();
        model
            .add_multiple_regulations(vec!["a -> b", "b -> a", "a -| a"])
            .unwrap();
        model
            .add_empty_uninterpreted_fn_by_str("f", "f", 2)
            .unwrap();
        model.set_update_fn(&var_a, "(b & !a) | f(a, b)").unwrap();
        model
            .set_uninterpreted_fn_expression_by_str("f", "var0 => var1")
            .unwrap();
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

    /// Test whether conversion to BN correctly propagates expressions of uninterpreted
    /// functions into update functions.
    ///
    /// Update fn for `a` is `(b & !a) | f(a, b)`, which contains uninterpreted fn `f`.
    /// We set expression for `f(var0, var1)` to `var0 => var1`. Therefore, the update
    /// function should be transformed into `(b & !a) | (a => b)`
    #[test]
    fn test_to_bn_with_propagated_expressions() {
        let mut model = prepare_test_model_full();
        model
            .set_uninterpreted_fn_expression_by_str("f", "var0 => var1")
            .unwrap();

        let bn = model.to_bn();
        let var_a = bn.as_graph().find_variable("a").unwrap();
        let var_b = bn.as_graph().find_variable("b").unwrap();
        let update_var_a = bn
            .get_update_function(var_a)
            .as_ref()
            .unwrap()
            .to_string(&bn);
        assert_eq!(update_var_a, "(b & !a) | (a => b)");
        assert_eq!(bn.get_update_function(var_b), &None);
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
