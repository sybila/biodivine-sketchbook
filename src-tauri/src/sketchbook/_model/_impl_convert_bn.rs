use crate::sketchbook::ModelState;
use biodivine_lib_param_bn::{BooleanNetwork, RegulatoryGraph};

/// Methods for converting between `ModelState` and `BooleanNetwork` (from the `lib-param-bn`).
impl ModelState {
    /// Convert the `ModelState` into the corresponding "default" `BooleanNetwork` object.
    /// The resulting BN covers the variables and regulations, but it has empty update functions,
    /// and does not cover parameters.
    pub fn to_empty_bn(&self) -> BooleanNetwork {
        let reg_graph = self.to_reg_graph();
        BooleanNetwork::new(reg_graph)
    }

    /// Convert the `ModelState` into the corresponding "default" `BooleanNetwork` object with added
    /// parameters.
    /// The resulting BN covers the variables, parameters, and regulations, but it has empty update functions.
    pub fn to_empty_bn_with_params(&self) -> BooleanNetwork {
        let mut bn = self.to_empty_bn();
        for (fn_id, uninterpreted_fn) in self.uninterpreted_fns.iter() {
            // uninterpreted fns always have unique valid IDs, so we can unwrap
            bn.add_parameter(fn_id.as_str(), uninterpreted_fn.get_arity() as u32)
                .unwrap();
        }
        bn
    }

    /// Generate a `BooleanNetwork` with a only given number of "fake" variables.
    /// These variables will be named `var0`, `var1`, ...
    ///
    /// The resulting BN will cover all parameters of this model.
    /// There will be no regulations, and update functions will be empty.
    ///
    /// This is useful for parsing a `FnUpdate` objects describing structure of uninterpreted functions.
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
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::ModelState;

    #[test]
    fn test_to_empty_bn_with_params() {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        model
            .add_multiple_regulations(vec!["a -> b", "b -> a"])
            .unwrap();
        model.add_uninterpreted_fn_by_str("f", "f", 3).unwrap();

        let bn = model.to_empty_bn_with_params();
        let var_a = bn.as_graph().find_variable("a");
        let var_b = bn.as_graph().find_variable("b");
        assert_eq!(bn.num_vars(), 2);
        assert_eq!(bn.regulators(var_a.unwrap()), vec![var_b.unwrap()]);
        assert_eq!(bn.num_parameters(), 1);
        assert!(bn.find_parameter("f").is_some());
    }
}
