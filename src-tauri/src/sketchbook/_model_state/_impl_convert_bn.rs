use crate::sketchbook::ModelState;
use biodivine_lib_param_bn::BooleanNetwork;

/// Methods for converting between `ModelState` and `BooleanNetwork` (from the `lib-param-bn`).
impl ModelState {
    /// Convert the `ModelState` into the corresponding "default" `BooleanNetwork` object.
    /// The resulting BN covers the variables and regulations, but it has empty update functions,
    /// and does not cover parameters.
    pub fn to_empty_bn(&self) -> BooleanNetwork {
        let reg_graph = self.to_reg_graph();
        BooleanNetwork::new(reg_graph)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::ModelState;

    #[test]
    fn test_to_empty_bn() {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        model
            .add_multiple_regulations(vec!["a -> b", "b -> a"])
            .unwrap();
        model.add_uninterpreted_fn_by_str("f", "f", 3).unwrap();

        let bn = model.to_empty_bn();
        let var_a = bn.as_graph().find_variable("a");
        let var_b = bn.as_graph().find_variable("b");
        assert_eq!(bn.num_vars(), 2);
        assert_eq!(bn.regulators(var_a.unwrap()), vec![var_b.unwrap()]);
        assert_eq!(bn.num_parameters(), 0);
    }
}
