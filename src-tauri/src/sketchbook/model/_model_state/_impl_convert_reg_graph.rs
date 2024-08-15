use crate::sketchbook::bn_utils;
use crate::sketchbook::ids::VarId;
use crate::sketchbook::model::ModelState;
use biodivine_lib_param_bn::RegulatoryGraph;

/// Methods for converting between `ModelState` and `RegulatoryGraph` (from the `lib-param-bn`).
impl ModelState {
    /// Extract the regulatory graph (`RegulatoryGraph` object) from the `ModelState`.
    /// Sorted variable IDs of the `ModelState` are used for variable names in `RegulatoryGraph`.
    ///
    /// The conversion might loose some information, as the `RegulatoryGraph` does not support
    /// all the variants of `Monotonicity` and `Essentiality`. See also [bn_utils::sign_to_monotonicity].
    ///
    /// Note that we can convert the resulting `RegulatoryGraph` back, but the conversion loses
    /// some information, like the original variable names and layout information.
    /// Also, all of the other model components, such as `update functions` or `uninterpreted functions`
    /// are not part of the `RegulatoryGraph`.
    pub fn to_reg_graph(&self) -> RegulatoryGraph {
        self._to_reg_graph(true)
    }

    /// Extract the regulatory graph (`RegulatoryGraph` object) from the `ModelState`.
    /// Sorted variable IDs of the `ModelState` are used for variable names in `RegulatoryGraph`.
    ///
    /// The types of regulations (their essentiality and monotonicity) are ignored, and unspecified
    /// versions are used instead.
    ///
    /// This might be useful in inference, if we want to process regulation types later via static
    /// properties.
    pub fn to_reg_graph_with_unspecified_regs(&self) -> RegulatoryGraph {
        self._to_reg_graph(false)
    }

    /// Internal utility to extract the regulatory graph (`RegulatoryGraph` object) from the
    /// `ModelState`. Sorted variable IDs of the `ModelState` are used for variable names in
    /// `RegulatoryGraph`.
    ///
    /// There are two modes based on `include_reg_types` argument. If it is set to `true`, the
    /// types of regulations (their essentiality and monotonicity) are preserved. If it is set to
    /// `false`, they are ignored, and unspecified versions are used instead.
    fn _to_reg_graph(&self, include_reg_types: bool) -> RegulatoryGraph {
        // create `RegulatoryGraph` from a list of variable ID strings (these are unique and
        // can be mapped back)
        let mut variable_vec = self
            .variables
            .keys()
            .map(|v| v.to_string())
            .collect::<Vec<_>>();
        // sort the IDs, so that the process is kinda deterministic - in `RegulatoryGraph`, the
        // order of the variables matters (but regulations order does not)
        variable_vec.sort();
        let mut reg_graph = RegulatoryGraph::new(variable_vec);

        // regulations
        for r in self.regulations() {
            if include_reg_types {
                // add the regulation with its original monotonicity and essentiality (as far as conversion allows)
                reg_graph
                    .add_regulation(
                        r.get_regulator().as_str(),
                        r.get_target().as_str(),
                        r.is_essential(),
                        bn_utils::sign_to_monotonicity(r.get_sign()),
                    )
                    .unwrap();
                // we can use unwrap, cause the regulation is ensured to be unique and correctly added
            } else {
                // add the regulation unspecified monotonicity and essentiality
                reg_graph
                    .add_regulation(
                        r.get_regulator().as_str(),
                        r.get_target().as_str(),
                        false,
                        None,
                    )
                    .unwrap();
                // we can use unwrap, cause the regulation is ensured to be unique and correctly added
            }
        }
        reg_graph
    }

    /// Convert the `RegulatoryGraph` into the corresponding `ModelState` instance. A name
    /// of the variable used in `RegulatoryGraph` (which should be unique) is used as both its ID
    /// and name in the resulting `ModelState`.
    ///
    /// Note that only the default layout (all nodes at 0,0) is created for the `ModelState`.
    pub fn from_reg_graph(reg_graph: &RegulatoryGraph) -> Result<ModelState, String> {
        let mut model = ModelState::new_empty();

        // variables
        for v in reg_graph.variables() {
            // name in the `RegulatoryGraph` is a unique valid identifier
            let name_in_graph = reg_graph.get_variable_name(v);
            model.add_var(VarId::new(name_in_graph.as_str())?, name_in_graph)?;
        }

        // regulations
        for r in reg_graph.regulations() {
            let name_regulator = reg_graph.get_variable_name(r.get_regulator());
            let name_target = reg_graph.get_variable_name(r.get_target());
            model.add_regulation(
                VarId::new(name_regulator.as_str())?,
                VarId::new(name_target.as_str())?,
                bn_utils::essentiality_from_bool(r.is_observable()),
                bn_utils::sign_from_monotonicity(r.get_monotonicity()),
            )?;
        }
        Ok(model)
    }
}

#[cfg(test)]
mod tests {
    use crate::sketchbook::model::ModelState;
    use biodivine_lib_param_bn::RegulatoryGraph;

    /// Prepare a test model containing only variables and regulations.
    fn prepare_test_model() -> ModelState {
        let mut model = ModelState::new_from_vars(vec![("a", "a"), ("b", "b")]).unwrap();
        model
            .add_multiple_regulations(vec!["a -> b", "b -> a", "a -| a"])
            .unwrap();
        model
    }

    #[test]
    fn test_to_reg_graph() {
        let model = prepare_test_model();
        let reg_graph = model.to_reg_graph();
        let var_a = reg_graph.find_variable("a").unwrap();
        let var_b = reg_graph.find_variable("b").unwrap();
        assert_eq!(reg_graph.num_vars(), 2);
        assert_eq!(reg_graph.regulators(var_a), vec![var_a, var_b]);

        let model_back = ModelState::from_reg_graph(&reg_graph).unwrap();
        assert_eq!(model, model_back);
    }

    #[test]
    fn test_from_reg_graph() {
        let mut reg_graph = RegulatoryGraph::new(vec!["a".to_string(), "b".to_string()]);
        reg_graph.add_string_regulation("a -> b").unwrap();
        reg_graph.add_string_regulation("b -> a").unwrap();

        let model = ModelState::from_reg_graph(&reg_graph).unwrap();
        assert_eq!(model.num_vars(), 2);

        let reg_graph_back = model.to_reg_graph();
        assert_eq!(reg_graph, reg_graph_back);
    }
}
